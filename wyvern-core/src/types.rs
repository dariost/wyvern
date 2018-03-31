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

use builder::{ProgramBuilder, ProgramObjectInfo, WorkerMessage};
use num::Integer;
use program::TokenId;
use program::{ConstantScalar, DataType, Op, TokenType};
use std::cmp::{PartialEq, PartialOrd};
use std::marker::PhantomData;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};
use std::string::ToString;

pub trait Type: Copy + PartialEq + Default {
    type TrueType;
    fn data_type() -> DataType;
    fn symbol_constant(&self) -> ConstantScalar;
}

#[derive(Clone, Copy)]
pub struct Constant<'a, T: Type> {
    phantom: PhantomData<T>,
    pub(crate) info: ProgramObjectInfo<'a>,
}

#[derive(Clone, Copy)]
pub struct Variable<'a, T: Type> {
    phantom: PhantomData<T>,
    pub(crate) info: ProgramObjectInfo<'a>,
    ty: VariableType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VariableType {
    Variable,
    ArrayIndex(TokenId, TokenId),
}

#[derive(Clone, Copy)]
pub struct Array<'a, T: Type> {
    phantom: PhantomData<T>,
    pub(crate) info: ProgramObjectInfo<'a>,
}

macro_rules! impl_type {
    ($primitive:ty, $upper:ident) => {
        impl Type for $primitive {
            type TrueType = $primitive;
            fn data_type() -> DataType {
                DataType::$upper
            }
            fn symbol_constant(&self) -> ConstantScalar {
                ConstantScalar::$upper(*self)
            }
        }
    };
}

macro_rules! impl_unary_op {
    ($lower:ident, $upper:ident) => {
        impl<'a, T: Type + $upper> $upper for Constant<'a, T> {
            type Output = Constant<'a, T>;

            fn $lower(self) -> Self::Output {
                let result = Self::generate(self.info.builder);
                result
                    .info
                    .builder
                    .add_operation(Op::$upper(result.info.token.id, self.info.token.id));
                result
            }
        }
    };
}

macro_rules! impl_binary_op {
    ($lower:ident, $upper:ident) => {
        impl<'a, T: Type + $upper> $upper for Constant<'a, T> {
            type Output = Constant<'a, T>;

            fn $lower(self, rhs: Self) -> Self::Output {
                assert_eq!(self.info.builder, rhs.info.builder);
                let result = Self::generate(self.info.builder);
                result.info.builder.add_operation(Op::$upper(
                    result.info.token.id,
                    self.info.token.id,
                    rhs.info.token.id,
                ));
                result
            }
        }
    };
}

macro_rules! impl_binary_comparison_op {
    ($lower:ident, $upper:ident, $category:ident) => {
        impl<'a, T: Type + $category> Constant<'a, T> {
            pub fn $lower(self, rhs: Self) -> Constant<'a, bool> {
                assert_eq!(self.info.builder, rhs.info.builder);
                let result = Constant::<'a, bool>::generate(self.info.builder);
                result.info.builder.add_operation(Op::$upper(
                    result.info.token.id,
                    self.info.token.id,
                    rhs.info.token.id,
                ));
                result
            }
        }
    };
}

macro_rules! impl_binary_op_immediate {
    ($lower:ident, $upper:ident) => {
        impl<'a, T: Type + $upper> $upper<T> for Constant<'a, T> {
            type Output = Constant<'a, T>;

            fn $lower(self, rhs: T) -> Self::Output {
                let immediate = Constant::new(rhs, self.info.builder);
                let result = Self::generate(self.info.builder);
                result.info.builder.add_operation(Op::$upper(
                    result.info.token.id,
                    self.info.token.id,
                    immediate.info.token.id,
                ));
                result
            }
        }
    };
}

macro_rules! impl_shift_op {
    ($lower:ident, $upper:ident) => {
        impl<'a, U: Type + Integer, T: Type + $upper<U>> $upper<Constant<'a, U>>
            for Constant<'a, T>
        {
            type Output = Constant<'a, T>;

            fn $lower(self, rhs: Constant<'a, U>) -> Self::Output {
                assert_eq!(self.info.builder, rhs.info.builder);
                let result = Self::generate(self.info.builder);
                result.info.builder.add_operation(Op::$upper(
                    result.info.token.id,
                    self.info.token.id,
                    rhs.info.token.id,
                ));
                result
            }
        }
    };
}

macro_rules! impl_shift_op_immediate {
    ($lower:ident, $upper:ident) => {
        impl<'a, U: Type + Integer, T: Type + $upper<U>> $upper<U> for Constant<'a, T> {
            type Output = Constant<'a, T>;

            fn $lower(self, rhs: U) -> Self::Output {
                let immediate = Constant::new(rhs, self.info.builder);
                let result = Self::generate(self.info.builder);
                result.info.builder.add_operation(Op::$upper(
                    result.info.token.id,
                    self.info.token.id,
                    immediate.info.token.id,
                ));
                result
            }
        }
    };
}

macro_rules! impl_conversion {
    ($lower1:ident, $lower2:ident, $conv:ident) => {
        impl<'a> From<Constant<'a, $lower1>> for Constant<'a, $lower2> {
            fn from(obj: Constant<'a, $lower1>) -> Constant<'a, $lower2> {
                let result = Constant::generate(obj.info.builder);
                result
                    .info
                    .builder
                    .add_operation(Op::$conv(result.info.token.id, obj.info.token.id));
                result
            }
        }
    };
}

impl<'a, T: Type> Constant<'a, T> {
    pub fn new(value: T, builder: &'a ProgramBuilder) -> Constant<'a, T> {
        let constant = Self::generate(builder);
        constant.info.builder.add_operation(Op::Constant(
            constant.info.token.id,
            value.symbol_constant(),
        ));
        constant
    }

    pub(crate) fn generate(builder: &'a ProgramBuilder) -> Constant<'a, T> {
        Constant {
            phantom: PhantomData,
            info: builder.gen_token(TokenType::Constant(T::data_type())),
        }
    }
}

impl<'a, T: Type> Variable<'a, T> {
    pub fn new(builder: &'a ProgramBuilder) -> Variable<'a, T> {
        Variable {
            phantom: PhantomData,
            info: builder.gen_token(TokenType::Variable(T::data_type())),
            ty: VariableType::Variable,
        }
    }

    pub fn load(&self) -> Constant<'a, T> {
        match self.ty {
            VariableType::Variable => {
                let result = Constant::generate(self.info.builder);
                result
                    .info
                    .builder
                    .add_operation(Op::Load(result.info.token.id, self.info.token.id));
                result
            }
            VariableType::ArrayIndex(array, index) => {
                let result = Constant::generate(self.info.builder);
                self.info
                    .builder
                    .add_operation(Op::ArrayLoad(result.info.token.id, array, index));
                result
            }
        }
    }

    pub fn store(&self, object: Constant<'a, T>) {
        assert_eq!(self.info.builder, object.info.builder);
        match self.ty {
            VariableType::Variable => {
                self.info
                    .builder
                    .add_operation(Op::Store(self.info.token.id, object.info.token.id));
            }
            VariableType::ArrayIndex(array, index) => {
                self.info
                    .builder
                    .add_operation(Op::ArrayStore(array, index, object.info.token.id));
            }
        }
    }

    pub fn mark_as_input<S: ToString>(&self, name: S) -> Self {
        if self.ty != VariableType::Variable {
            panic!("Can't mark array index as input");
        }
        let name = name;
        self.info.builder.send_message(WorkerMessage::MarkInput(
            self.info.token.id,
            name.to_string(),
        ));
        *self
    }

    pub fn mark_as_output<S: ToString>(&self, name: S) -> Self {
        if self.ty != VariableType::Variable {
            panic!("Can't mark array index as output");
        }
        let name = name;
        self.info.builder.send_message(WorkerMessage::MarkOutput(
            self.info.token.id,
            name.to_string(),
        ));
        *self
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(len_without_is_empty))]
impl<'a, T: Type> Array<'a, T> {
    pub fn new<U: Type + Integer>(
        size: Constant<'a, U>,
        builder: &'a ProgramBuilder,
    ) -> Array<'a, T> {
        assert_eq!(builder, size.info.builder);
        let result = Array {
            phantom: PhantomData,
            info: builder.gen_token(TokenType::Array(T::data_type())),
        };
        result
            .info
            .builder
            .add_operation(Op::ArrayNew(result.info.token.id, size.info.token.id));
        result
    }

    pub fn mark_as_input<S: ToString>(&self, name: S) -> Self {
        let name = name;
        self.info.builder.send_message(WorkerMessage::MarkInput(
            self.info.token.id,
            name.to_string(),
        ));
        *self
    }

    pub fn mark_as_output<S: ToString>(&self, name: S) -> Self {
        let name = name;
        self.info.builder.send_message(WorkerMessage::MarkOutput(
            self.info.token.id,
            name.to_string(),
        ));
        *self
    }

    pub fn len(&'a self) -> Constant<'a, u32> {
        let result = Constant::generate(self.info.builder);
        result
            .info
            .builder
            .add_operation(Op::ArrayLen(result.info.token.id, self.info.token.id));
        result
    }

    pub fn at<U: Type + Integer>(&'a self, index: Constant<'a, U>) -> Variable<'a, T> {
        assert_eq!(self.info.builder, index.info.builder);
        Variable {
            phantom: PhantomData,
            info: self.info
                .builder
                .gen_token(TokenType::ArrayPointer(T::data_type())),
            ty: VariableType::ArrayIndex(self.info.token.id, index.info.token.id),
        }
    }
}

impl_type!(i32, I32);
impl_type!(u32, U32);
impl_type!(f32, F32);
impl_type!(bool, Bool);

impl_binary_op!(add, Add);
impl_binary_op!(sub, Sub);
impl_binary_op!(mul, Mul);
impl_binary_op!(div, Div);
impl_binary_op!(rem, Rem);
impl_unary_op!(neg, Neg);
impl_unary_op!(not, Not);
impl_shift_op!(shl, Shl);
impl_shift_op!(shr, Shr);
impl_binary_op!(bitand, BitAnd);
impl_binary_op!(bitor, BitOr);
impl_binary_op!(bitxor, BitXor);

impl_binary_op_immediate!(add, Add);
impl_binary_op_immediate!(sub, Sub);
impl_binary_op_immediate!(mul, Mul);
impl_binary_op_immediate!(div, Div);
impl_binary_op_immediate!(rem, Rem);
impl_shift_op_immediate!(shl, Shl);
impl_shift_op_immediate!(shr, Shr);
impl_binary_op_immediate!(bitand, BitAnd);
impl_binary_op_immediate!(bitor, BitOr);
impl_binary_op_immediate!(bitxor, BitXor);

impl_binary_comparison_op!(eq, Eq, PartialEq);
impl_binary_comparison_op!(ne, Ne, PartialEq);
impl_binary_comparison_op!(lt, Lt, PartialOrd);
impl_binary_comparison_op!(le, Le, PartialOrd);
impl_binary_comparison_op!(gt, Gt, PartialOrd);
impl_binary_comparison_op!(ge, Ge, PartialOrd);

impl_conversion!(i32, u32, U32fromI32);
impl_conversion!(i32, f32, F32fromI32);
impl_conversion!(u32, i32, I32fromU32);
impl_conversion!(u32, f32, F32fromU32);
impl_conversion!(f32, i32, I32fromF32);
impl_conversion!(f32, u32, U32fromF32);
