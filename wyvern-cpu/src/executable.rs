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

use resource::CpuResource;
use std::collections::HashMap;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};
use std::sync::Arc;
use wcore::executor::{Executable, Resource, IO};
use wcore::program::{ConstantScalar, Op, Program, TokenId, TokenValue};

#[derive(Debug)]
pub struct CpuExecutable {
    pub(crate) program: Program,
    pub(crate) binding: HashMap<(IO, String), Arc<CpuResource>>,
}

impl Executable for CpuExecutable {
    type Error = String;
    type Report = String;
    type Resource = CpuResource;

    fn bind<S: ToString>(
        &mut self,
        name: S,
        kind: IO,
        resource: Arc<CpuResource>,
    ) -> Option<Arc<Self::Resource>> {
        self.binding.insert((kind, name.to_string()), resource)
    }

    fn unbind<S: ToString>(&mut self, name: S, kind: IO) -> Option<Arc<Self::Resource>> {
        self.binding.remove(&(kind, name.to_string()))
    }

    fn run(&mut self) -> Result<String, String> {
        self.simulate()
    }
}

impl CpuExecutable {
    fn simulate(&mut self) -> Result<String, String> {
        let mut memory = HashMap::new();
        for (name, id) in &self.program.input {
            let value = self.binding
                .get(&(IO::Input, name.clone()))
                .ok_or(format!("Missing input {}", name))?;
            memory.insert(*id, value.get_data());
        }
        let operations = self.program.operation.clone();
        self.run_block(&operations, &mut memory)?;
        for (name, id) in &self.program.output {
            let value = memory.remove(id).ok_or(format!("Missing output {}", name))?;
            self.binding
                .get(&(IO::Output, name.clone()))
                .ok_or(format!("Missing output {}", name))?
                .set_data(value);
        }
        Ok("Completed!".into())
    }

    fn run_block(
        &mut self,
        block: &Vec<Op>,
        memory: &mut HashMap<TokenId, TokenValue>,
    ) -> Result<(), String> {
        for op in block {
            match *op {
                Op::Block(ref block) => self.run_block(block, memory)?,
                Op::MemoryBarrier => continue,
                Op::ControlBarrier => continue,
                Op::WorkerId(r) => Self::insert_scalar(memory, r, ConstantScalar::U32(0)),
                Op::NumWorkers(r) => Self::insert_scalar(memory, r, ConstantScalar::U32(1)),
                Op::Load(r, a) => {
                    let v = Self::get_scalar(memory, a)?;
                    Self::insert_scalar(memory, r, v)
                }
                Op::Store(r, a) => {
                    let v = Self::get_scalar(memory, a)?;
                    Self::insert_scalar(memory, r, v)
                }
                Op::Constant(r, a) => Self::insert_scalar(memory, r, a),
                Op::U32fromF32(r, a) => {
                    let v = Self::get_f32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::U32(v as u32))
                }
                Op::I32fromF32(r, a) => {
                    let v = Self::get_f32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::I32(v as i32))
                }
                Op::F32fromU32(r, a) => {
                    let v = Self::get_u32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::F32(v as f32))
                }
                Op::F32fromI32(r, a) => {
                    let v = Self::get_i32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::F32(v as f32))
                }
                Op::I32fromU32(r, a) => {
                    let v = Self::get_u32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::I32(v as i32))
                }
                Op::U32fromI32(r, a) => {
                    let v = Self::get_i32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::U32(v as u32))
                }
                Op::Add(r, a, b) => Self::op_add(memory, r, a, b)?,
                Op::Sub(r, a, b) => Self::op_sub(memory, r, a, b)?,
                Op::Mul(r, a, b) => Self::op_mul(memory, r, a, b)?,
                Op::Div(r, a, b) => Self::op_div(memory, r, a, b)?,
                Op::Rem(r, a, b) => Self::op_rem(memory, r, a, b)?,
                Op::Neg(r, a) => Self::op_neg(memory, r, a)?,
                Op::Not(r, a) => Self::op_not(memory, r, a)?,
                Op::Shl(r, a, b) => Self::op_shl(memory, r, a, b)?,
                Op::Shr(r, a, b) => Self::op_shr(memory, r, a, b)?,
                Op::BitAnd(r, a, b) => Self::op_bitand(memory, r, a, b)?,
                Op::BitOr(r, a, b) => Self::op_bitor(memory, r, a, b)?,
                Op::BitXor(r, a, b) => Self::op_bitxor(memory, r, a, b)?,
            }
        }
        Ok(())
    }

    fn get_scalar(
        memory: &HashMap<TokenId, TokenValue>,
        id: TokenId,
    ) -> Result<ConstantScalar, String> {
        let value = memory.get(&id).ok_or(format!("{:?} doesn't exist!", id))?;
        match *value {
            TokenValue::Scalar(x) => Ok(x),
            _ => unreachable!(),
        }
    }

    fn insert_scalar(
        memory: &mut HashMap<TokenId, TokenValue>,
        id: TokenId,
        value: ConstantScalar,
    ) {
        memory.insert(id, TokenValue::Scalar(value));
    }

    fn op_neg(
        memory: &mut HashMap<TokenId, TokenValue>,
        r: TokenId,
        a: TokenId,
    ) -> Result<(), String> {
        let a = Self::get_scalar(memory, a)?;
        let v = match a {
            ConstantScalar::I32(x) => ConstantScalar::I32(Neg::neg(x)),
            ConstantScalar::F32(x) => ConstantScalar::F32(Neg::neg(x)),
            _ => unreachable!(),
        };
        Self::insert_scalar(memory, r, v);
        Ok(())
    }

    fn op_not(
        memory: &mut HashMap<TokenId, TokenValue>,
        r: TokenId,
        a: TokenId,
    ) -> Result<(), String> {
        let a = Self::get_scalar(memory, a)?;
        let v = match a {
            ConstantScalar::U32(x) => ConstantScalar::U32(Not::not(x)),
            ConstantScalar::I32(x) => ConstantScalar::I32(Not::not(x)),
            ConstantScalar::Bool(x) => ConstantScalar::Bool(Not::not(x)),
            _ => unreachable!(),
        };
        Self::insert_scalar(memory, r, v);
        Ok(())
    }
}

macro_rules! impl_get_type {
    ($lower: ty, $upper: ident, $fn_name: ident) => {
        impl CpuExecutable {
            fn $fn_name(
                memory: &HashMap<TokenId, TokenValue>,
                id: TokenId,
            ) -> Result<$lower, String> {
                let value = Self::get_scalar(memory, id)?;
                match value {
                    ConstantScalar::$upper(x) => Ok(x),
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! impl_binary_op {
    ($fn_name: ident, $lower: ident, $upper: ident) => {
        impl CpuExecutable {
            fn $fn_name(
                memory: &mut HashMap<TokenId, TokenValue>,
                r: TokenId,
                a: TokenId,
                b: TokenId,
            ) -> Result<(), String> {
                let a = Self::get_scalar(memory, a)?;
                let b = Self::get_scalar(memory, b)?;
                let v = match (a, b) {
                    (ConstantScalar::U32(x), ConstantScalar::U32(y)) => {
                        ConstantScalar::U32($upper::$lower(x, y))
                    }
                    (ConstantScalar::I32(x), ConstantScalar::I32(y)) => {
                        ConstantScalar::I32($upper::$lower(x, y))
                    }
                    (ConstantScalar::F32(x), ConstantScalar::F32(y)) => {
                        ConstantScalar::F32($upper::$lower(x, y))
                    }
                    _ => unreachable!(),
                };
                Self::insert_scalar(memory, r, v);
                Ok(())
            }
        }
    };
}

macro_rules! impl_binary_shift_op {
    ($fn_name: ident, $lower: ident, $upper: ident) => {
        impl CpuExecutable {
            fn $fn_name(
                memory: &mut HashMap<TokenId, TokenValue>,
                r: TokenId,
                a: TokenId,
                b: TokenId,
            ) -> Result<(), String> {
                let a = Self::get_scalar(memory, a)?;
                let b = Self::get_scalar(memory, b)?;
                let v = match (a, b) {
                    (ConstantScalar::U32(x), ConstantScalar::U32(y)) => {
                        ConstantScalar::U32($upper::$lower(x, y))
                    }
                    (ConstantScalar::I32(x), ConstantScalar::U32(y)) => {
                        ConstantScalar::I32($upper::$lower(x, y))
                    }
                    (ConstantScalar::U32(x), ConstantScalar::I32(y)) => {
                        ConstantScalar::U32($upper::$lower(x, y))
                    }
                    (ConstantScalar::I32(x), ConstantScalar::I32(y)) => {
                        ConstantScalar::I32($upper::$lower(x, y))
                    }
                    _ => unreachable!(),
                };
                Self::insert_scalar(memory, r, v);
                Ok(())
            }
        }
    };
}

macro_rules! impl_binary_binary_op {
    ($fn_name: ident, $lower: ident, $upper: ident) => {
        impl CpuExecutable {
            fn $fn_name(
                memory: &mut HashMap<TokenId, TokenValue>,
                r: TokenId,
                a: TokenId,
                b: TokenId,
            ) -> Result<(), String> {
                let a = Self::get_scalar(memory, a)?;
                let b = Self::get_scalar(memory, b)?;
                let v = match (a, b) {
                    (ConstantScalar::U32(x), ConstantScalar::U32(y)) => {
                        ConstantScalar::U32($upper::$lower(x, y))
                    }
                    (ConstantScalar::I32(x), ConstantScalar::I32(y)) => {
                        ConstantScalar::I32($upper::$lower(x, y))
                    }
                    (ConstantScalar::Bool(x), ConstantScalar::Bool(y)) => {
                        ConstantScalar::Bool($upper::$lower(x, y))
                    }
                    _ => unreachable!(),
                };
                Self::insert_scalar(memory, r, v);
                Ok(())
            }
        }
    };
}

impl_get_type!(u32, U32, get_u32);
impl_get_type!(i32, I32, get_i32);
impl_get_type!(f32, F32, get_f32);

impl_binary_op!(op_add, add, Add);
impl_binary_op!(op_sub, sub, Sub);
impl_binary_op!(op_mul, mul, Mul);
impl_binary_op!(op_div, div, Div);
impl_binary_op!(op_rem, rem, Rem);
impl_binary_binary_op!(op_bitand, bitand, BitAnd);
impl_binary_binary_op!(op_bitor, bitor, BitOr);
impl_binary_binary_op!(op_bitxor, bitxor, BitXor);
impl_binary_shift_op!(op_shl, shl, Shl);
impl_binary_shift_op!(op_shr, shr, Shr);
