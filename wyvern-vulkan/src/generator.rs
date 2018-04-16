// Copyright 2018 | Dario Ostuni <dario.ostuni@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use rspirv::binary::Assemble;
use rspirv::mr::{Builder, Operand};
use spirv_headers::{AddressingModel, Capability, ExecutionMode, ExecutionModel, MemoryModel};
use spirv_headers::{BuiltIn, Decoration, FunctionControl, StorageClass, Word};
use std::collections::HashMap;
use wcore::executor::IO;
use wcore::program::{ConstantScalar, DataType, LabelId, Op, Program, TokenId, TokenType};

type Binding = (usize, Option<(IO, String)>);

#[allow(non_snake_case)]
pub fn generate(program: &Program) -> Result<(Vec<u32>, Vec<Binding>), String> {
    const LOCAL_SIZE: u32 = 1;
    let mut b = Builder::new();
    b.capability(Capability::Shader);
    let gl_std = b.ext_inst_import("GLSL.std.450");
    b.memory_model(AddressingModel::Logical, MemoryModel::GLSL450);
    let main_function = b.id();
    let global_invocation_id = b.id();
    let num_work_groups = b.id();
    b.entry_point(
        ExecutionModel::GLCompute,
        main_function,
        "main",
        &[global_invocation_id, num_work_groups],
    );
    b.execution_mode(main_function, ExecutionMode::LocalSize, &[LOCAL_SIZE, 1, 1]);
    #[allow(dead_code)]
    struct Types {
        type_void: Word,
        type_bool: Word,
        type_u32: Word,
        type_i32: Word,
        type_f32: Word,
        type_v3u32: Word,
        type_inu32: Word,
        type_inv3u32: Word,
    }
    struct Constants {
        CONSTANT_0: Word,
        SCOPE_DEVICE: Word,
        SCOPE_WORKGROUP: Word,
        SEMANTIC_ACQUIRERELEASE: Word,
        LOCAL_SIZE_WORD: Word,
    }
    let type_void = b.type_void();
    let type_bool = b.type_bool();
    let type_u32 = b.type_int(32, 0);
    let type_i32 = b.type_int(32, 1);
    let type_f32 = b.type_float(32);
    let type_v3u32 = b.type_vector(type_u32, 3);
    let type_inu32 = b.type_pointer(None, StorageClass::Input, type_u32);
    let type_inv3u32 = b.type_pointer(None, StorageClass::Input, type_v3u32);
    let ty = Types {
        type_void,
        type_bool,
        type_u32,
        type_i32,
        type_f32,
        type_v3u32,
        type_inu32,
        type_inv3u32,
    };
    b.variable(
        ty.type_inv3u32,
        Some(global_invocation_id),
        StorageClass::Input,
        None,
    );
    b.variable(
        ty.type_inv3u32,
        Some(num_work_groups),
        StorageClass::Input,
        None,
    );
    let CONSTANT_0 = b.constant_u32(ty.type_u32, 0);
    let SCOPE_DEVICE = b.constant_u32(ty.type_u32, 1);
    let SCOPE_WORKGROUP = b.constant_u32(ty.type_u32, 2);
    let SEMANTIC_ACQUIRERELEASE = b.constant_u32(ty.type_u32, 0x8 | 0x40);
    let LOCAL_SIZE_WORD = b.constant_u32(ty.type_u32, LOCAL_SIZE);
    let cn = Constants {
        CONSTANT_0,
        SCOPE_DEVICE,
        SCOPE_WORKGROUP,
        SEMANTIC_ACQUIRERELEASE,
        LOCAL_SIZE_WORD,
    };
    let type_main_function = b.type_function(ty.type_void, &[]);
    b.decorate(
        global_invocation_id,
        Decoration::BuiltIn,
        &[Operand::BuiltIn(BuiltIn::GlobalInvocationId)],
    );
    b.decorate(
        num_work_groups,
        Decoration::BuiltIn,
        &[Operand::BuiltIn(BuiltIn::NumWorkgroups)],
    );
    let _main_function = b.begin_function(
        ty.type_void,
        Some(main_function),
        FunctionControl::empty(),
        type_main_function,
    ).map_err(|x| format!("{:?}", x))?;
    let mut token_map = HashMap::new();
    let mut label_map = HashMap::new();
    for symbol in program.symbol.keys() {
        token_map.insert(*symbol, b.id());
    }
    label_map.insert(
        LabelId(0),
        b.begin_basic_block(None).map_err(|x| format!("{:?}", x))?,
    );
    let global_invocation_id_var =
        b.access_chain(ty.type_inu32, None, global_invocation_id, &[cn.CONSTANT_0])
            .map_err(|x| format!("{:?}", x))?;
    let num_work_groups_var =
        b.access_chain(ty.type_inu32, None, num_work_groups, &[cn.CONSTANT_0])
            .map_err(|x| format!("{:?}", x))?;
    let global_invocation_id_word = b.load(ty.type_u32, None, global_invocation_id_var, None, &[])
        .map_err(|x| format!("{:?}", x))?;
    let num_workers_word = b.load(ty.type_u32, None, num_work_groups_var, None, &[])
        .map_err(|x| format!("{:?}", x))?;
    let num_workers_word = b.imul(ty.type_u32, None, num_workers_word, cn.LOCAL_SIZE_WORD)
        .map_err(|x| format!("{:?}", x))?;
    struct Words {
        worker_id: Word,
        num_workers: Word,
    }
    let w = Words {
        worker_id: global_invocation_id_word,
        num_workers: num_workers_word,
    };
    fn compile(
        operations: &[Op],
        b: &mut Builder,
        program: &Program,
        ty: &Types,
        cn: &Constants,
        w: &Words,
        token_map: &mut HashMap<TokenId, Word>,
        label_map: &mut HashMap<LabelId, Word>,
    ) -> Result<(), String> {
        let get_const_type = |x: TokenId| match program.symbol[&x] {
            TokenType::Constant(DataType::Bool) => ty.type_bool,
            TokenType::Constant(DataType::U32) => ty.type_u32,
            TokenType::Constant(DataType::I32) => ty.type_i32,
            TokenType::Constant(DataType::F32) => ty.type_f32,
            _ => unreachable!(),
        };
        let get_const_datatype = |x: TokenId| match program.symbol[&x] {
            TokenType::Constant(x) => x,
            _ => unreachable!(),
        };
        for op in operations {
            match *op {
                Op::MemoryBarrier => {
                    b.memory_barrier(cn.SCOPE_DEVICE, cn.SEMANTIC_ACQUIRERELEASE)
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::ControlBarrier => {
                    b.control_barrier(
                        cn.SCOPE_WORKGROUP,
                        cn.SCOPE_DEVICE,
                        cn.SEMANTIC_ACQUIRERELEASE,
                    ).map_err(|x| format!("{:?}", x))?;
                }
                Op::WorkerId(r) => {
                    token_map.insert(r, w.worker_id);
                }
                Op::NumWorkers(r) => {
                    token_map.insert(r, w.num_workers);
                }
                Op::Constant(r, a) => {
                    if let TokenType::Constant(t) = program.symbol[&r] {
                        let new_tokenid = match (t, a) {
                            (DataType::Bool, ConstantScalar::Bool(a)) => {
                                if a {
                                    b.constant_true(ty.type_bool)
                                } else {
                                    b.constant_false(ty.type_bool)
                                }
                            }
                            (DataType::U32, ConstantScalar::U32(a)) => {
                                b.constant_u32(ty.type_u32, a)
                            }
                            (DataType::F32, ConstantScalar::F32(a)) => {
                                b.constant_f32(ty.type_f32, a)
                            }
                            (DataType::I32, ConstantScalar::I32(a)) => {
                                b.constant_u32(ty.type_i32, a as u32)
                            }
                            _ => unreachable!(),
                        };
                        token_map.insert(r, new_tokenid);
                    } else {
                        unreachable!();
                    };
                }
                Op::U32fromF32(r, a) => {
                    b.convert_fto_u(ty.type_u32, Some(token_map[&r]), token_map[&a])
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::I32fromF32(r, a) => {
                    b.convert_fto_s(ty.type_i32, Some(token_map[&r]), token_map[&a])
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::F32fromU32(r, a) => {
                    b.convert_uto_f(ty.type_f32, Some(token_map[&r]), token_map[&a])
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::F32fromI32(r, a) => {
                    b.convert_sto_f(ty.type_f32, Some(token_map[&r]), token_map[&a])
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::I32fromU32(r, a) => {
                    b.bitcast(ty.type_i32, Some(token_map[&r]), token_map[&a])
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::U32fromI32(r, a) => {
                    b.bitcast(ty.type_u32, Some(token_map[&r]), token_map[&a])
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::Add(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.iadd(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.iadd(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.fadd(
                                ty.type_f32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Sub(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.isub(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.isub(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.fsub(
                                ty.type_f32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Mul(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.imul(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.imul(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.fmul(
                                ty.type_f32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Div(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.udiv(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.sdiv(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.fdiv(
                                ty.type_f32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Rem(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.umod(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.smod(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.fmod(
                                ty.type_f32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Neg(r, a) => {
                    match get_const_datatype(r) {
                        DataType::U32 => unreachable!(),
                        DataType::I32 => {
                            b.snegate(ty.type_i32, Some(token_map[&r]), token_map[&a])
                                .map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.fnegate(ty.type_f32, Some(token_map[&r]), token_map[&a])
                                .map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Not(r, a) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.not(ty.type_u32, Some(token_map[&r]), token_map[&a])
                                .map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.not(ty.type_i32, Some(token_map[&r]), token_map[&a])
                                .map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => unreachable!(),
                        DataType::Bool => {
                            b.logical_not(ty.type_bool, Some(token_map[&r]), token_map[&a])
                                .map_err(|x| format!("{:?}", x))?;
                        }
                    };
                }
                Op::Shl(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.shift_left_logical(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.shift_left_logical(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => unreachable!(),
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Shr(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.shift_right_logical(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.shift_right_logical(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => unreachable!(),
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::BitAnd(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.bitwise_and(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.bitwise_and(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => unreachable!(),
                        DataType::Bool => {
                            b.logical_and(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                    };
                }
                Op::BitOr(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.bitwise_or(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.bitwise_or(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => unreachable!(),
                        DataType::Bool => {
                            b.logical_or(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                    };
                }
                Op::BitXor(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.bitwise_xor(
                                ty.type_u32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.bitwise_xor(
                                ty.type_i32,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => unreachable!(),
                        DataType::Bool => {
                            let na = b.logical_not(ty.type_bool, None, token_map[&a])
                                .map_err(|x| format!("{:?}", x))?;
                            let a = token_map[&a];
                            let nd = b.logical_not(ty.type_bool, None, token_map[&d])
                                .map_err(|x| format!("{:?}", x))?;
                            let d = token_map[&d];
                            let p1 = b.logical_and(ty.type_bool, None, a, nd)
                                .map_err(|x| format!("{:?}", x))?;
                            let p2 = b.logical_and(ty.type_bool, None, na, d)
                                .map_err(|x| format!("{:?}", x))?;
                            b.logical_or(ty.type_bool, Some(token_map[&r]), p1, p2)
                                .map_err(|x| format!("{:?}", x))?;
                        }
                    };
                }
                Op::Eq(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 | DataType::I32 => {
                            b.iequal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.ford_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => {
                            b.logical_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                    };
                }
                Op::Ne(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 | DataType::I32 => {
                            b.inot_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.ford_not_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => {
                            b.logical_not_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                    };
                }
                Op::Lt(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.uless_than(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.sless_than(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.ford_less_than(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Le(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.uless_than_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.sless_than_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.ford_less_than_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Gt(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.ugreater_than(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.sgreater_than(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.ford_greater_than(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Ge(r, a, d) => {
                    match get_const_datatype(r) {
                        DataType::U32 => {
                            b.ugreater_than_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::I32 => {
                            b.sgreater_than_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::F32 => {
                            b.ford_greater_than_equal(
                                ty.type_bool,
                                Some(token_map[&r]),
                                token_map[&a],
                                token_map[&d],
                            ).map_err(|x| format!("{:?}", x))?;
                        }
                        DataType::Bool => unreachable!(),
                    };
                }
                Op::Phi(r, a0, l0, a1, l1) => {
                    b.phi(
                        get_const_type(r),
                        Some(token_map[&r]),
                        &[
                            (token_map[&a0], label_map[&l0]),
                            (token_map[&a1], label_map[&l1]),
                        ],
                    ).map_err(|x| format!("{:?}", x))?;
                }
                Op::If(ref cond_op, cond, l0, ref a0, lend) => unimplemented!(),
                Op::IfElse(ref cond_op, cond, l0, ref a0, l1, ref a1, lend) => unimplemented!(),
                Op::While(lcond, ref cond_op, cond, l0, ref a0, lend) => unimplemented!(),
                Op::Load(r, a) => unimplemented!(),
                Op::Store(r, a) => unimplemented!(),
                Op::ArrayNew(r, s, t, ms, shared) => unimplemented!(),
                Op::ArrayLen(r, v) => unimplemented!(),
                Op::ArrayStore(v, i, a) => unimplemented!(),
                Op::ArrayLoad(r, v, i) => unimplemented!(),
            };
        }
        Ok(())
    }
    compile(
        &program.operation,
        &mut b,
        &program,
        &ty,
        &cn,
        &w,
        &mut token_map,
        &mut label_map,
    )?;
    // END
    b.ret().map_err(|x| format!("{:?}", x))?;
    b.end_function().map_err(|x| format!("{:?}", x))?;
    Ok((b.module().assemble(), Vec::new()))
}
