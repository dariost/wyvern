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

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub symbol: HashMap<TokenId, TokenType>,
    pub operation: Vec<Op>,
    pub input: HashMap<String, TokenId>,
    pub output: HashMap<String, TokenId>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Constant(DataType),
    Variable(DataType),
    ArrayPointer(DataType),
    Array(DataType),
    Null,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataType {
    Bool,
    I32,
    U32,
    F32,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenValue {
    Scalar(ConstantScalar),
    Vector(ConstantVector),
    Null,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConstantScalar {
    Bool(bool),
    I32(i32),
    U32(u32),
    F32(f32),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstantVector {
    Bool(Vec<bool>),
    I32(Vec<i32>),
    U32(Vec<u32>),
    F32(Vec<f32>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct TokenId(u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token {
    pub(crate) id: TokenId,
    pub(crate) ty: TokenType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Block(Vec<Op>),
    MemoryBarrier,
    ControlBarrier,
    WorkerId(TokenId),
    NumWorkers(TokenId),
    Load(TokenId, TokenId),
    Store(TokenId, TokenId),
    ArrayNew(TokenId, TokenId),
    ArrayLen(TokenId, TokenId),
    ArrayLoad(TokenId, TokenId, TokenId),
    ArrayStore(TokenId, TokenId, TokenId),
    Constant(TokenId, ConstantScalar),
    U32fromF32(TokenId, TokenId),
    I32fromF32(TokenId, TokenId),
    F32fromU32(TokenId, TokenId),
    F32fromI32(TokenId, TokenId),
    I32fromU32(TokenId, TokenId),
    U32fromI32(TokenId, TokenId),
    Add(TokenId, TokenId, TokenId),
    Sub(TokenId, TokenId, TokenId),
    Mul(TokenId, TokenId, TokenId),
    Div(TokenId, TokenId, TokenId),
    Rem(TokenId, TokenId, TokenId),
    Neg(TokenId, TokenId),
    Not(TokenId, TokenId),
    Shl(TokenId, TokenId, TokenId),
    Shr(TokenId, TokenId, TokenId),
    BitAnd(TokenId, TokenId, TokenId),
    BitOr(TokenId, TokenId, TokenId),
    BitXor(TokenId, TokenId, TokenId),
    Eq(TokenId, TokenId, TokenId),
    Ne(TokenId, TokenId, TokenId),
    Lt(TokenId, TokenId, TokenId),
    Le(TokenId, TokenId, TokenId),
    Gt(TokenId, TokenId, TokenId),
    Ge(TokenId, TokenId, TokenId),
}

impl TokenId {
    pub(crate) fn next(&mut self) -> TokenId {
        let prev = *self;
        self.0 += 1;
        prev
    }
}

pub fn get_token_type(value: &TokenValue) -> TokenType {
    match *value {
        TokenValue::Null => TokenType::Null,
        TokenValue::Scalar(ref x) => TokenType::Variable(match *x {
            ConstantScalar::Bool(_) => DataType::Bool,
            ConstantScalar::I32(_) => DataType::I32,
            ConstantScalar::U32(_) => DataType::U32,
            ConstantScalar::F32(_) => DataType::F32,
        }),
        TokenValue::Vector(ref x) => TokenType::Array(match *x {
            ConstantVector::Bool(_) => DataType::Bool,
            ConstantVector::I32(_) => DataType::I32,
            ConstantVector::U32(_) => DataType::U32,
            ConstantVector::F32(_) => DataType::F32,
        }),
    }
}
