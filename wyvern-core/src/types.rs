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

use builder::{ProgramBuilder, ProgramObjectInfo};
use std::marker::PhantomData;

pub trait Type {}

#[derive(Clone, Copy)]
pub struct Constant<'a, T: Type> {
    phantom: PhantomData<T>,
    info: ProgramObjectInfo<'a>,
}

impl<'a, T: Type> Constant<'a, T> {
    pub fn new(value: T, builder: &'a ProgramBuilder) -> Constant<'a, T> {
        Constant {
            phantom: PhantomData,
            info: builder.gen_poi(),
        }
    }
}
