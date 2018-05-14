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

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use wcore::executor::Resource;
use wcore::program::{ConstantScalar, ConstantVector, DataType, TokenType, TokenValue};

pub(crate) enum ResourceType {
    Empty,
    U32(CpuAccessibleBuffer<u32>),
    I32(CpuAccessibleBuffer<i32>),
    F32(CpuAccessibleBuffer<f32>),
    VU32(CpuAccessibleBuffer<[u32]>),
    VI32(CpuAccessibleBuffer<[i32]>),
    VF32(CpuAccessibleBuffer<[f32]>),
}

pub struct VkResource {
    pub(crate) id: u32,
    pub(crate) resource: Mutex<ResourceType>,
}

impl Resource for VkResource {
    fn clear(&self) {
        *self.resource.lock().unwrap() = ResourceType::Empty;
    }

    fn token_type(&self) -> TokenType {
        match *self.resource.lock().unwrap() {
            ResourceType::Empty => TokenType::Null,
            ResourceType::U32(_) => TokenType::Variable(DataType::U32),
            ResourceType::I32(_) => TokenType::Variable(DataType::I32),
            ResourceType::F32(_) => TokenType::Variable(DataType::F32),
            ResourceType::VU32(_) => TokenType::Array(DataType::U32),
            ResourceType::VI32(_) => TokenType::Array(DataType::I32),
            ResourceType::VF32(_) => TokenType::Array(DataType::F32),
        }
    }

    fn set_data(&self, value: TokenValue) {
        unimplemented!();
    }

    fn get_data(&self) -> TokenValue {
        match *self.resource.lock().unwrap() {
            ResourceType::Empty => TokenValue::Null,
            ResourceType::U32(ref v) => TokenValue::Scalar(ConstantScalar::U32(*v.read().unwrap())),
            ResourceType::I32(ref v) => TokenValue::Scalar(ConstantScalar::I32(*v.read().unwrap())),
            ResourceType::F32(ref v) => TokenValue::Scalar(ConstantScalar::F32(*v.read().unwrap())),
            ResourceType::VU32(_) => unimplemented!(),
            ResourceType::VI32(_) => unimplemented!(),
            ResourceType::VF32(_) => unimplemented!(),
        }
    }
}

impl PartialEq for VkResource {
    fn eq(&self, other: &VkResource) -> bool {
        self.id == other.id
    }
}

impl Eq for VkResource {}

impl Hash for VkResource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Debug for VkResource {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        write!(f, "VkResource {{id: {}}}", self.id)
    }
}
