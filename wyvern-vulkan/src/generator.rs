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
use spirv_headers::LoopControl;
use spirv_headers::{AddressingModel, Capability, ExecutionMode, ExecutionModel, MemoryModel};
use spirv_headers::{BuiltIn, Decoration, FunctionControl, SelectionControl, StorageClass, Word};
use std::collections::{HashMap, HashSet};
use wcore::executor::IO;
use wcore::program::StorageType;
use wcore::program::{ConstantScalar, DataType, LabelId, Op, Program, TokenId, TokenType};

#[derive(Debug, Clone)]
pub enum BindType {
    Public(IO, String),
    Private(u32),
}

pub type Binding = (u32, BindType, bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VkVersion {
    Vulkan10,
    Vulkan11,
}

#[allow(non_snake_case)]
#[cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]
pub fn generate(program: &Program, version: VkVersion) -> Result<(Vec<u32>, Vec<Binding>), String> {
    const LOCAL_SIZE: u32 = 1;
    let mut next_binding = 0;
    let mut bindings = Vec::new();
    let mut b = Builder::new();
    match version {
        VkVersion::Vulkan10 => b.set_version(1, 0),
        VkVersion::Vulkan11 => b.set_version(1, 3),
    };
    b.capability(Capability::Shader);
    match version {
        VkVersion::Vulkan10 => {}
        VkVersion::Vulkan11 => {
            b.extension("SPV_KHR_storage_buffer_storage_class");
            b.extension("SPV_KHR_variable_pointers");
        }
    };
    #[allow(unused_variables)]
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
    struct Types {
        type_void: Word,
        type_bool: Word,
        type_u32: Word,
        type_i32: Word,
        type_f32: Word,
        type_funbool: Word,
        type_funu32: Word,
        type_funi32: Word,
        type_funf32: Word,
        type_stbool: Word,
        type_stu32: Word,
        type_sti32: Word,
        type_stf32: Word,
        #[allow(dead_code)]
        type_v3u32: Word,
        type_inu32: Word,
        type_inv3u32: Word,
    }
    struct Constants {
        CONSTANT_0: Word,
        CONSTANT_1: Word,
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
    let type_funbool = b.type_pointer(None, StorageClass::Function, type_bool);
    let type_funu32 = b.type_pointer(None, StorageClass::Function, type_u32);
    let type_funi32 = b.type_pointer(None, StorageClass::Function, type_i32);
    let type_funf32 = b.type_pointer(None, StorageClass::Function, type_f32);
    let stclass = match version {
        VkVersion::Vulkan10 => StorageClass::Uniform,
        VkVersion::Vulkan11 => StorageClass::StorageBuffer,
    };
    let type_stbool = b.type_pointer(None, stclass, type_bool);
    let type_stu32 = b.type_pointer(None, stclass, type_u32);
    let type_sti32 = b.type_pointer(None, stclass, type_i32);
    let type_stf32 = b.type_pointer(None, stclass, type_f32);
    let type_v3u32 = b.type_vector(type_u32, 3);
    let type_inu32 = b.type_pointer(None, StorageClass::Input, type_u32);
    let type_inv3u32 = b.type_pointer(None, StorageClass::Input, type_v3u32);
    let ty = Types {
        type_void,
        type_bool,
        type_u32,
        type_i32,
        type_f32,
        type_funbool,
        type_funu32,
        type_funi32,
        type_funf32,
        type_stbool,
        type_stu32,
        type_sti32,
        type_stf32,
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
    let CONSTANT_1 = b.constant_u32(ty.type_u32, 1);
    let SCOPE_DEVICE = b.constant_u32(ty.type_u32, 1);
    let SCOPE_WORKGROUP = b.constant_u32(ty.type_u32, 2);
    let SEMANTIC_ACQUIRERELEASE = b.constant_u32(ty.type_u32, 0x8 | 0x40);
    let LOCAL_SIZE_WORD = b.constant_u32(ty.type_u32, LOCAL_SIZE);
    let cn = Constants {
        CONSTANT_0,
        CONSTANT_1,
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
    let mut token_map = HashMap::new();
    let mut label_map = HashMap::new();
    let mut in_set = HashMap::new();
    let mut out_set = HashMap::new();
    for symbol in program.symbol.keys() {
        token_map.insert(*symbol, b.id());
    }
    for k in program.input.keys() {
        in_set.insert(program.input[k], k.clone());
    }
    for k in program.output.keys() {
        out_set.insert(program.output[k], k.clone());
    }
    let mut st_set = HashSet::new();
    for t in program.storage.keys() {
        let input = in_set.contains_key(t);
        let output = out_set.contains_key(t);
        match (program.storage[&t], input || output) {
            (StorageType::Variable(tty), true) => {
                let binding_number = next_binding;
                next_binding += 1;
                let struct_type = b.type_struct(&[match tty {
                    DataType::Bool => ty.type_bool,
                    DataType::U32 => ty.type_u32,
                    DataType::I32 => ty.type_i32,
                    DataType::F32 => ty.type_f32,
                }]);
                let struct_type_pointer = b.type_pointer(
                    None,
                    match version {
                        VkVersion::Vulkan10 => StorageClass::Uniform,
                        VkVersion::Vulkan11 => StorageClass::StorageBuffer,
                    },
                    struct_type,
                );
                let struct_instance = b.variable(
                    struct_type_pointer,
                    Some(token_map[&t]),
                    match version {
                        VkVersion::Vulkan10 => StorageClass::Uniform,
                        VkVersion::Vulkan11 => StorageClass::StorageBuffer,
                    },
                    None,
                );
                b.member_decorate(
                    struct_type,
                    0,
                    Decoration::Offset,
                    &[Operand::LiteralInt32(0)],
                );
                b.decorate(
                    struct_type,
                    match version {
                        VkVersion::Vulkan10 => Decoration::BufferBlock,
                        VkVersion::Vulkan11 => Decoration::Block,
                    },
                    &[],
                );
                b.decorate(
                    struct_instance,
                    Decoration::DescriptorSet,
                    &[Operand::LiteralInt32(0)],
                );
                b.decorate(
                    struct_instance,
                    Decoration::Binding,
                    &[Operand::LiteralInt32(binding_number)],
                );
                if input {
                    bindings.push((
                        binding_number,
                        BindType::Public(IO::Input, in_set[t].clone()),
                        false,
                    ))
                }
                if output {
                    bindings.push((
                        binding_number,
                        BindType::Public(IO::Output, out_set[t].clone()),
                        false,
                    ))
                }
            }
            (StorageType::SharedArray(tty, ms), io) => {
                st_set.insert(*t);
                let binding_number = next_binding;
                next_binding += 1;
                let array_type = b.type_runtime_array(match tty {
                    DataType::Bool => return Err("bool I/O is not supported".into()),
                    DataType::U32 => ty.type_u32,
                    DataType::I32 => ty.type_i32,
                    DataType::F32 => ty.type_f32,
                });
                let offset = match tty {
                    DataType::Bool => return Err("bool I/O is not supported".into()),
                    DataType::U32 | DataType::I32 | DataType::F32 => 4,
                };
                let struct_type = b.type_struct(&[ty.type_u32, array_type]);
                let struct_type_pointer = b.type_pointer(
                    None,
                    match version {
                        VkVersion::Vulkan10 => StorageClass::Uniform,
                        VkVersion::Vulkan11 => StorageClass::StorageBuffer,
                    },
                    struct_type,
                );
                let struct_instance = b.variable(
                    struct_type_pointer,
                    Some(token_map[&t]),
                    match version {
                        VkVersion::Vulkan10 => StorageClass::Uniform,
                        VkVersion::Vulkan11 => StorageClass::StorageBuffer,
                    },
                    None,
                );
                b.decorate(
                    array_type,
                    Decoration::ArrayStride,
                    &[Operand::LiteralInt32(offset)],
                );
                b.member_decorate(
                    struct_type,
                    0,
                    Decoration::Offset,
                    &[Operand::LiteralInt32(0)],
                );
                b.member_decorate(
                    struct_type,
                    1,
                    Decoration::Offset,
                    &[Operand::LiteralInt32(4)],
                );
                b.decorate(
                    struct_type,
                    match version {
                        VkVersion::Vulkan10 => Decoration::BufferBlock,
                        VkVersion::Vulkan11 => Decoration::Block,
                    },
                    &[],
                );
                b.decorate(
                    struct_instance,
                    Decoration::DescriptorSet,
                    &[Operand::LiteralInt32(0)],
                );
                b.decorate(
                    struct_instance,
                    Decoration::Binding,
                    &[Operand::LiteralInt32(binding_number)],
                );
                if io {
                    if input {
                        bindings.push((
                            binding_number,
                            BindType::Public(IO::Input, in_set[t].clone()),
                            true,
                        ))
                    }
                    if output {
                        bindings.push((
                            binding_number,
                            BindType::Public(IO::Output, out_set[t].clone()),
                            true,
                        ))
                    }
                } else {
                    bindings.push((binding_number, BindType::Private(ms), true));
                }
            }
            _ => {}
        };
    }
    let _main_function = b.begin_function(
        ty.type_void,
        Some(main_function),
        FunctionControl::empty(),
        type_main_function,
    ).map_err(|x| format!("{:?}", x))?;
    label_map.insert(
        LabelId(0),
        b.begin_basic_block(None).map_err(|x| format!("{:?}", x))?,
    );
    for t in program.storage.keys() {
        if in_set.contains_key(&t) || out_set.contains_key(&t) {
            if let StorageType::Variable(tty) = program.storage[&t] {
                let new_token = b.access_chain(
                    match tty {
                        DataType::Bool => ty.type_stbool,
                        DataType::U32 => ty.type_stu32,
                        DataType::I32 => ty.type_sti32,
                        DataType::F32 => ty.type_stf32,
                    },
                    None,
                    token_map[&t],
                    &[cn.CONSTANT_0],
                ).map_err(|x| format!("{:?}", x))?;
                let result = token_map.insert(*t, new_token);
                assert!(result.is_some());
            }
            continue;
        }
        match program.storage[&t] {
            StorageType::Variable(tty) => {
                b.variable(
                    match tty {
                        DataType::Bool => ty.type_funbool,
                        DataType::U32 => ty.type_funu32,
                        DataType::I32 => ty.type_funi32,
                        DataType::F32 => ty.type_funf32,
                    },
                    Some(token_map[&t]),
                    StorageClass::Function,
                    None,
                );
            }
            StorageType::PrivateArray(tty, ms) => {
                let array_max_size = b.constant_u32(ty.type_u32, ms);
                let array_type = b.type_array(
                    match tty {
                        DataType::Bool => ty.type_funbool,
                        DataType::U32 => ty.type_funu32,
                        DataType::I32 => ty.type_funi32,
                        DataType::F32 => ty.type_funf32,
                    },
                    array_max_size,
                );
                let storage_type = b.type_struct(&[ty.type_u32, array_type]);
                let storage_pointer_type =
                    b.type_pointer(None, StorageClass::Function, storage_type);
                b.variable(
                    storage_pointer_type,
                    Some(token_map[&t]),
                    StorageClass::Function,
                    None,
                );
            }
            StorageType::SharedArray(_, _) => {}
        };
    }
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
    #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments, many_single_char_names))]
    fn compile(
        operations: &[Op],
        b: &mut Builder,
        program: &Program,
        ty: &Types,
        cn: &Constants,
        w: &Words,
        token_map: &mut HashMap<TokenId, Word>,
        label_map: &mut HashMap<LabelId, Word>,
        st_set: &HashSet<TokenId>,
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
        let get_array_type = |x: TokenId, io: bool| match (program.symbol[&x], io) {
            (TokenType::Array(DataType::Bool), false) => ty.type_funbool,
            (TokenType::Array(DataType::U32), false) => ty.type_funu32,
            (TokenType::Array(DataType::I32), false) => ty.type_funi32,
            (TokenType::Array(DataType::F32), false) => ty.type_funf32,
            (TokenType::Array(DataType::Bool), true) => ty.type_stbool,
            (TokenType::Array(DataType::U32), true) => ty.type_stu32,
            (TokenType::Array(DataType::I32), true) => ty.type_sti32,
            (TokenType::Array(DataType::F32), true) => ty.type_stf32,
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
                    match get_const_datatype(a) {
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
                    match get_const_datatype(a) {
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
                    match get_const_datatype(a) {
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
                    match get_const_datatype(a) {
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
                    match get_const_datatype(a) {
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
                    match get_const_datatype(a) {
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
                Op::If(ref cond_op, cond, l0, ref a0, lend) => {
                    let l0 = *label_map.entry(l0).or_insert_with(|| b.id());
                    let lend = *label_map.entry(lend).or_insert_with(|| b.id());
                    compile(cond_op, b, program, ty, cn, w, token_map, label_map, st_set)?;
                    b.selection_merge(lend, SelectionControl::NONE)
                        .map_err(|x| format!("{:?}", x))?;
                    b.branch_conditional(token_map[&cond], l0, lend, &[])
                        .map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(l0))
                        .map_err(|x| format!("{:?}", x))?;
                    compile(a0, b, program, ty, cn, w, token_map, label_map, st_set)?;
                    b.branch(lend).map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(lend))
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::IfElse(ref cond_op, cond, l0, ref a0, l1, ref a1, lend) => {
                    let l0 = *label_map.entry(l0).or_insert_with(|| b.id());
                    let l1 = *label_map.entry(l1).or_insert_with(|| b.id());
                    let lend = *label_map.entry(lend).or_insert_with(|| b.id());
                    compile(cond_op, b, program, ty, cn, w, token_map, label_map, st_set)?;
                    b.selection_merge(lend, SelectionControl::NONE)
                        .map_err(|x| format!("{:?}", x))?;
                    b.branch_conditional(token_map[&cond], l0, l1, &[])
                        .map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(l0))
                        .map_err(|x| format!("{:?}", x))?;
                    compile(a0, b, program, ty, cn, w, token_map, label_map, st_set)?;
                    b.branch(lend).map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(l1))
                        .map_err(|x| format!("{:?}", x))?;
                    compile(a1, b, program, ty, cn, w, token_map, label_map, st_set)?;
                    b.branch(lend).map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(lend))
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::While(lcond, ref cond_op, cond, l0, ref a0, lend) => {
                    let lbefore = b.id();
                    let lcond = *label_map.entry(lcond).or_insert_with(|| b.id());
                    let l0 = *label_map.entry(l0).or_insert_with(|| b.id());
                    let lcontinue = b.id();
                    let lend = *label_map.entry(lend).or_insert_with(|| b.id());
                    b.branch(lbefore).map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(lbefore))
                        .map_err(|x| format!("{:?}", x))?;
                    b.loop_merge(lend, lcontinue, LoopControl::NONE, &[])
                        .map_err(|x| format!("{:?}", x))?;
                    b.branch(lcond).map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(lcond))
                        .map_err(|x| format!("{:?}", x))?;
                    compile(cond_op, b, program, ty, cn, w, token_map, label_map, st_set)?;
                    b.branch_conditional(token_map[&cond], l0, lend, &[])
                        .map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(l0))
                        .map_err(|x| format!("{:?}", x))?;
                    compile(a0, b, program, ty, cn, w, token_map, label_map, st_set)?;
                    b.branch(lcontinue).map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(lcontinue))
                        .map_err(|x| format!("{:?}", x))?;
                    b.branch(lbefore).map_err(|x| format!("{:?}", x))?;
                    b.begin_basic_block(Some(lend))
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::Load(r, a) => {
                    b.load(
                        get_const_type(r),
                        Some(token_map[&r]),
                        token_map[&a],
                        None,
                        &[],
                    ).map_err(|x| format!("{:?}", x))?;
                }
                Op::Store(r, a) => {
                    b.store(token_map[&r], token_map[&a], None, &[])
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::ArrayNew(r, s, _, _, _) => {
                    if st_set.contains(&r) {
                        continue;
                    }
                    let size_pointer =
                        b.access_chain(ty.type_funu32, None, token_map[&r], &[cn.CONSTANT_0])
                            .map_err(|x| format!("{:?}", x))?;
                    b.store(size_pointer, token_map[&s], None, &[])
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::ArrayLen(r, v) => {
                    let size_pointer = b.access_chain(
                        if st_set.contains(&v) {
                            ty.type_stu32
                        } else {
                            ty.type_funu32
                        },
                        None,
                        token_map[&v],
                        &[cn.CONSTANT_0],
                    ).map_err(|x| format!("{:?}", x))?;
                    b.load(
                        get_const_type(r),
                        Some(token_map[&r]),
                        size_pointer,
                        None,
                        &[],
                    ).map_err(|x| format!("{:?}", x))?;
                }
                Op::ArrayStore(v, i, a) => {
                    let pointer = b.access_chain(
                        get_array_type(v, st_set.contains(&v)),
                        None,
                        token_map[&v],
                        &[cn.CONSTANT_1, token_map[&i]],
                    ).map_err(|x| format!("{:?}", x))?;
                    b.store(pointer, token_map[&a], None, &[])
                        .map_err(|x| format!("{:?}", x))?;
                }
                Op::ArrayLoad(r, v, i) => {
                    let pointer = b.access_chain(
                        get_array_type(v, st_set.contains(&v)),
                        None,
                        token_map[&v],
                        &[cn.CONSTANT_1, token_map[&i]],
                    ).map_err(|x| format!("{:?}", x))?;
                    b.load(get_const_type(r), Some(token_map[&r]), pointer, None, &[])
                        .map_err(|x| format!("{:?}", x))?;
                }
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
        &st_set,
    )?;
    b.ret().map_err(|x| format!("{:?}", x))?;
    b.end_function().map_err(|x| format!("{:?}", x))?;
    Ok((b.module().assemble(), bindings))
}
