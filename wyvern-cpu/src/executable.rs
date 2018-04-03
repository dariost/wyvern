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
use wcore::program::DataType;
use wcore::program::{ConstantScalar, ConstantVector, LabelId, Op, Program, TokenId, TokenValue};

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
                .ok_or_else(|| format!("Missing input {}", name))?;
            memory.insert(*id, value.get_data());
        }
        let operations = self.program.operation.clone();
        let mut last_labels = (LabelId::default(), LabelId::default());
        self.run_block(&operations, &mut memory, &mut last_labels)?;
        for (name, id) in &self.program.output {
            let value = memory
                .remove(id)
                .ok_or_else(|| format!("Missing output {}", name))?;
            self.binding
                .get(&(IO::Output, name.clone()))
                .ok_or_else(|| format!("Missing output {}", name))?
                .set_data(value);
        }
        Ok("Completed!".into())
    }

    fn run_block(
        &mut self,
        block: &[Op],
        memory: &mut HashMap<TokenId, TokenValue>,
        labels: &mut (LabelId, LabelId),
    ) -> Result<(), String> {
        for op in block {
            match *op {
                Op::Phi(r, a0, l0, a1, l1) => {
                    let v;
                    if labels.0 == l0 {
                        v = Self::get_scalar(memory, a0)?;
                    } else if labels.0 == l1 {
                        v = Self::get_scalar(memory, a1)?;
                    } else {
                        unreachable!();
                    }
                    Self::insert_scalar(memory, r, v);
                }
                Op::If(ref cond_op, cond, l0, ref a0, lend) => {
                    self.run_block(cond_op, memory, labels)?;
                    let cond = Self::get_bool(memory, cond)?;
                    if cond {
                        Self::update_labels(labels, l0);
                        self.run_block(a0, memory, labels)?;
                    }
                    Self::update_labels(labels, lend);
                }
                Op::IfElse(ref cond_op, cond, l0, ref a0, l1, ref a1, lend) => {
                    self.run_block(cond_op, memory, labels)?;
                    let cond = Self::get_bool(memory, cond)?;
                    if cond {
                        Self::update_labels(labels, l0);
                        self.run_block(a0, memory, labels)?;
                    } else {
                        Self::update_labels(labels, l1);
                        self.run_block(a1, memory, labels)?;
                    }
                    Self::update_labels(labels, lend);
                }
                Op::While(lcond, ref cond_op, cond, l0, ref a0, lend) => {
                    Self::update_labels(labels, lcond);
                    loop {
                        self.run_block(cond_op, memory, labels)?;
                        let condition = Self::get_bool(memory, cond)?;
                        if !condition {
                            break;
                        }
                        Self::update_labels(labels, l0);
                        self.run_block(a0, memory, labels)?;
                    }
                    Self::update_labels(labels, lend);
                }
                Op::MemoryBarrier => continue,
                Op::ControlBarrier => continue,
                Op::WorkerId(r) => Self::insert_scalar(memory, r, ConstantScalar::U32(0)),
                Op::NumWorkers(r) => Self::insert_scalar(memory, r, ConstantScalar::U32(1)),
                Op::Load(r, a) => {
                    let v = Self::get_scalar(memory, a)?;
                    Self::insert_scalar(memory, r, v);
                }
                Op::Store(r, a) => {
                    let v = Self::get_scalar(memory, a)?;
                    Self::insert_scalar(memory, r, v);
                }
                Op::Constant(r, a) => Self::insert_scalar(memory, r, a),
                Op::U32fromF32(r, a) => {
                    let v = Self::get_f32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::U32(v as u32));
                }
                Op::I32fromF32(r, a) => {
                    let v = Self::get_f32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::I32(v as i32));
                }
                Op::F32fromU32(r, a) => {
                    let v = Self::get_u32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::F32(v as f32));
                }
                Op::F32fromI32(r, a) => {
                    let v = Self::get_i32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::F32(v as f32));
                }
                Op::I32fromU32(r, a) => {
                    let v = Self::get_u32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::I32(v as i32));
                }
                Op::U32fromI32(r, a) => {
                    let v = Self::get_i32(memory, a)?;
                    Self::insert_scalar(memory, r, ConstantScalar::U32(v as u32));
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
                Op::ArrayNew(r, s, t, _) => {
                    let s = Self::get_u32(memory, s)?;
                    Self::new_vector(memory, r, s, t);
                }
                Op::ArrayLen(r, v) => {
                    let v = Self::get_vector_length(memory, v)?;
                    Self::insert_scalar(memory, r, ConstantScalar::U32(v));
                }
                Op::ArrayStore(v, i, a) => Self::insert_vector(memory, v, i, a)?,
                Op::ArrayLoad(r, v, i) => {
                    let v = Self::index_vector(memory, v, i)?;
                    Self::insert_scalar(memory, r, v);
                }
                Op::Eq(r, a, b) => Self::op_eq(memory, r, a, b)?,
                Op::Ne(r, a, b) => Self::op_ne(memory, r, a, b)?,
                Op::Lt(r, a, b) => Self::op_lt(memory, r, a, b)?,
                Op::Le(r, a, b) => Self::op_le(memory, r, a, b)?,
                Op::Gt(r, a, b) => Self::op_gt(memory, r, a, b)?,
                Op::Ge(r, a, b) => Self::op_ge(memory, r, a, b)?,
            }
        }
        Ok(())
    }

    fn update_labels(labels: &mut (LabelId, LabelId), new_label: LabelId) {
        (labels.0).0 = (labels.1).0;
        (labels.1).0 = new_label.0;
    }

    fn get_scalar(
        memory: &HashMap<TokenId, TokenValue>,
        id: TokenId,
    ) -> Result<ConstantScalar, String> {
        let value = memory
            .get(&id)
            .ok_or_else(|| format!("{:?} doesn't exist!", id))?;
        match *value {
            TokenValue::Scalar(x) => Ok(x),
            _ => unreachable!(),
        }
    }

    fn get_vector_mut(
        memory: &mut HashMap<TokenId, TokenValue>,
        v: TokenId,
    ) -> Result<&mut ConstantVector, String> {
        let value = memory
            .get_mut(&v)
            .ok_or_else(|| format!("{:?} doesn't exist!", v))?;
        match *value {
            TokenValue::Vector(ref mut x) => Ok(x),
            _ => unreachable!(),
        }
    }

    fn get_vector(
        memory: &HashMap<TokenId, TokenValue>,
        v: TokenId,
    ) -> Result<&ConstantVector, String> {
        let value = memory
            .get(&v)
            .ok_or_else(|| format!("{:?} doesn't exist!", v))?;
        match *value {
            TokenValue::Vector(ref x) => Ok(x),
            _ => unreachable!(),
        }
    }

    fn get_vector_length(memory: &HashMap<TokenId, TokenValue>, v: TokenId) -> Result<u32, String> {
        Ok(match *Self::get_vector(memory, v)? {
            ConstantVector::I32(ref x) => x.len(),
            ConstantVector::U32(ref x) => x.len(),
            ConstantVector::F32(ref x) => x.len(),
            ConstantVector::Bool(ref x) => x.len(),
        } as u32)
    }

    fn insert_vector(
        memory: &mut HashMap<TokenId, TokenValue>,
        v: TokenId,
        i: TokenId,
        a: TokenId,
    ) -> Result<(), String> {
        let a = Self::get_scalar(memory, a)?;
        let i = Self::get_u32(memory, i)? as usize;
        let v = Self::get_vector_mut(memory, v)?;
        match (v, a) {
            (&mut ConstantVector::I32(ref mut v), ConstantScalar::I32(a)) => v[i] = a,
            (&mut ConstantVector::U32(ref mut v), ConstantScalar::U32(a)) => v[i] = a,
            (&mut ConstantVector::F32(ref mut v), ConstantScalar::F32(a)) => v[i] = a,
            (&mut ConstantVector::Bool(ref mut v), ConstantScalar::Bool(a)) => v[i] = a,
            _ => unreachable!(),
        };
        Ok(())
    }

    fn index_vector(
        memory: &mut HashMap<TokenId, TokenValue>,
        v: TokenId,
        i: TokenId,
    ) -> Result<ConstantScalar, String> {
        let v = Self::get_vector(memory, v)?;
        let i = Self::get_u32(memory, i)? as usize;
        Ok(match *v {
            ConstantVector::I32(ref v) => ConstantScalar::I32(v[i]),
            ConstantVector::U32(ref v) => ConstantScalar::U32(v[i]),
            ConstantVector::F32(ref v) => ConstantScalar::F32(v[i]),
            ConstantVector::Bool(ref v) => ConstantScalar::Bool(v[i]),
        })
    }

    fn insert_scalar(
        memory: &mut HashMap<TokenId, TokenValue>,
        id: TokenId,
        value: ConstantScalar,
    ) {
        memory.insert(id, TokenValue::Scalar(value));
    }

    fn new_vector(memory: &mut HashMap<TokenId, TokenValue>, id: TokenId, s: u32, t: DataType) {
        let s = s as usize;
        match t {
            DataType::I32 => memory.insert(id, TokenValue::Vector(ConstantVector::I32(vec![0; s]))),
            DataType::U32 => memory.insert(id, TokenValue::Vector(ConstantVector::U32(vec![0; s]))),
            DataType::F32 => {
                memory.insert(id, TokenValue::Vector(ConstantVector::F32(vec![0.0; s])))
            }
            DataType::Bool => {
                memory.insert(id, TokenValue::Vector(ConstantVector::Bool(vec![false; s])))
            }
        };
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
    ($lower:ty, $upper:ident, $fn_name:ident) => {
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
    ($fn_name:ident, $lower:ident, $upper:ident) => {
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
    ($fn_name:ident, $lower:ident, $upper:ident) => {
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
    ($fn_name:ident, $lower:ident, $upper:ident) => {
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

macro_rules! impl_binary_ord_op {
    ($fn_name:ident, $lower:ident, $upper:ident) => {
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
                        ConstantScalar::Bool(PartialOrd::$lower(&x, &y))
                    }
                    (ConstantScalar::I32(x), ConstantScalar::I32(y)) => {
                        ConstantScalar::Bool(PartialOrd::$lower(&x, &y))
                    }
                    (ConstantScalar::F32(x), ConstantScalar::F32(y)) => {
                        ConstantScalar::Bool(PartialOrd::$lower(&x, &y))
                    }
                    _ => unreachable!(),
                };
                Self::insert_scalar(memory, r, v);
                Ok(())
            }
        }
    };
}

macro_rules! impl_binary_eq_op {
    ($fn_name:ident, $lower:ident, $upper:ident) => {
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
                        ConstantScalar::Bool(PartialEq::$lower(&x, &y))
                    }
                    (ConstantScalar::I32(x), ConstantScalar::I32(y)) => {
                        ConstantScalar::Bool(PartialEq::$lower(&x, &y))
                    }
                    (ConstantScalar::F32(x), ConstantScalar::F32(y)) => {
                        ConstantScalar::Bool(PartialEq::$lower(&x, &y))
                    }
                    (ConstantScalar::Bool(x), ConstantScalar::Bool(y)) => {
                        ConstantScalar::Bool(PartialEq::$lower(&x, &y))
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
impl_get_type!(bool, Bool, get_bool);

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
impl_binary_eq_op!(op_eq, eq, Eq);
impl_binary_eq_op!(op_ne, ne, Ne);
impl_binary_ord_op!(op_lt, lt, Lt);
impl_binary_ord_op!(op_le, le, Le);
impl_binary_ord_op!(op_gt, gt, Gt);
impl_binary_ord_op!(op_ge, ge, Ge);
