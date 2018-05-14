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

use std::f32;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::buffer::BufferUsage;
use vulkano::device::Device;
use wcore::executor::Resource;
use wcore::program::{ConstantScalar, ConstantVector, DataType, TokenType, TokenValue};

pub(crate) enum ResourceType {
    Empty,
    U32(Arc<CpuAccessibleBuffer<u32>>),
    I32(Arc<CpuAccessibleBuffer<i32>>),
    F32(Arc<CpuAccessibleBuffer<f32>>),
    VU32(Arc<CpuAccessibleBuffer<[u32]>>),
    VI32(Arc<CpuAccessibleBuffer<[i32]>>),
    VF32(Arc<CpuAccessibleBuffer<[f32]>>),
}

pub struct VkResource {
    pub(crate) id: u32,
    pub(crate) resource: Arc<Mutex<ResourceType>>,
    pub(crate) device: Arc<Device>,
}

impl VkResource {
    pub(crate) fn get_handle(&self) -> Arc<Mutex<ResourceType>> {
        self.resource.clone()
    }
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
        let resource = match value {
            TokenValue::Scalar(ConstantScalar::U32(x)) => ResourceType::U32(
                CpuAccessibleBuffer::from_data(self.device.clone(), BufferUsage::all(), x).unwrap(),
            ),
            TokenValue::Scalar(ConstantScalar::I32(x)) => ResourceType::I32(
                CpuAccessibleBuffer::from_data(self.device.clone(), BufferUsage::all(), x).unwrap(),
            ),
            TokenValue::Scalar(ConstantScalar::F32(x)) => ResourceType::F32(
                CpuAccessibleBuffer::from_data(self.device.clone(), BufferUsage::all(), x).unwrap(),
            ),
            TokenValue::Vector(ConstantVector::U32(mut x)) => ResourceType::VU32({
                x.insert(0, x.len() as u32);
                CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::all(),
                    x.into_iter(),
                ).unwrap()
            }),
            TokenValue::Vector(ConstantVector::I32(mut x)) => ResourceType::VI32({
                x.insert(0, x.len() as i32);
                CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::all(),
                    x.into_iter(),
                ).unwrap()
            }),
            TokenValue::Vector(ConstantVector::F32(mut x)) => ResourceType::VF32({
                x.insert(0, f32::from_bits(x.len() as u32));
                CpuAccessibleBuffer::from_iter(
                    self.device.clone(),
                    BufferUsage::all(),
                    x.into_iter(),
                ).unwrap()
            }),
            _ => panic!("Invalid TokenValue type"),
        };
        *self.resource.lock().unwrap() = resource;
    }

    fn get_data(&self) -> TokenValue {
        match *self.resource.lock().unwrap() {
            ResourceType::Empty => TokenValue::Null,
            ResourceType::U32(ref v) => TokenValue::Scalar(ConstantScalar::U32(*v.read().unwrap())),
            ResourceType::I32(ref v) => TokenValue::Scalar(ConstantScalar::I32(*v.read().unwrap())),
            ResourceType::F32(ref v) => TokenValue::Scalar(ConstantScalar::F32(*v.read().unwrap())),
            ResourceType::VU32(ref v) => TokenValue::Vector(ConstantVector::U32({
                let v = v.read().unwrap();
                let mut w = Vec::new();
                let s = v.len();
                #[cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]
                for i in 0..s {
                    w.push(v[i]);
                }
                let s = w.remove(0);
                w.truncate(s as usize);
                w
            })),
            ResourceType::VI32(ref v) => TokenValue::Vector(ConstantVector::I32({
                let v = v.read().unwrap();
                let mut w = Vec::new();
                let s = v.len();
                #[cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]
                for i in 0..s {
                    w.push(v[i]);
                }
                let s = w.remove(0);
                w.truncate(s as usize);
                w
            })),
            ResourceType::VF32(ref v) => TokenValue::Vector(ConstantVector::F32({
                let v = v.read().unwrap();
                let mut w = Vec::new();
                let s = v.len();
                #[cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]
                for i in 0..s {
                    w.push(v[i]);
                }
                let s = f32::to_bits(w.remove(0));
                w.truncate(s as usize);
                w
            })),
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
