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

use executor::ModuleLayout;
use generator::{BindType, Binding, VkVersion};
use resource::ResourceType;
use resource::VkResource;
use vulkano::buffer::BufferAccess;
use std::mem::swap;
use std::sync::Arc;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor::DescriptorDesc;
use vulkano::descriptor::descriptor_set::DescriptorWrite;
use vulkano::descriptor::descriptor_set::UnsafeDescriptorPool;
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSet;
use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
use vulkano::descriptor::descriptor_set::{DescriptorSet, DescriptorSetDesc};
use vulkano::descriptor::pipeline_layout::PipelineLayout;
use vulkano::descriptor::pipeline_layout::PipelineLayoutDesc;
use vulkano::device::{Device, Queue};
use vulkano::image::ImageViewAccess;
use vulkano::pipeline::shader::ShaderModule;
use vulkano::pipeline::ComputePipeline;
use wcore::executor::{Executable, IO};
use wcore::program::{DataType, Program};
use vulkano::sync::now;
use vulkano::sync::GpuFuture;

pub struct VkExecutable {
    pub(crate) module: Arc<ShaderModule>,
    pub(crate) program: Program,
    pub(crate) bindings: Vec<Binding>,
    pub(crate) assoc: Vec<Option<Arc<VkResource>>>,
    pub(crate) pool: UnsafeDescriptorPool,
    pub(crate) pipeline: Arc<ComputePipeline<PipelineLayout<ModuleLayout>>>,
    pub(crate) layout: ModuleLayout,
    pub(crate) device: Arc<Device>,
    pub(crate) queue: Arc<Queue>,
    pub(crate) version: VkVersion,
}

impl Executable for VkExecutable {
    type Error = String;
    type Report = String;
    type Resource = VkResource;

    fn bind<S: ToString>(
        &mut self,
        name: S,
        kind: IO,
        resource: Arc<VkResource>,
    ) -> Option<Arc<Self::Resource>> {
        let name = name.to_string();
        for b in &self.bindings {
            match b.1 {
                BindType::Public(k, ref n) if k == kind && *n == name => {
                    let mut result = Some(resource);
                    swap(&mut result, &mut self.assoc[b.0 as usize]);
                    return result;
                }
                _ => {}
            }
        }
        unreachable!();
    }

    fn unbind<S: ToString>(&mut self, name: S, kind: IO) -> Option<Arc<Self::Resource>> {
        let name = name.to_string();
        for b in &self.bindings {
            match b.1 {
                BindType::Public(k, ref n) if k == kind && *n == name => {
                    let mut result = None;
                    swap(&mut result, &mut self.assoc[b.0 as usize]);
                    return result;
                }
                _ => {}
            }
        }
        None
    }

    fn run(&mut self) -> Result<String, String> {
        let unsafe_layout_object = UnsafeDescriptorSetLayout::new(
            self.device.clone(),
            (0..self.layout.num_bindings_in_set(0).unwrap()).map(|x| self.layout.descriptor(0, x)),
        ).unwrap();
        let layout = (0..1).map(|_| &unsafe_layout_object);
        let mut set = unsafe { self.pool.alloc(layout) }.unwrap().next().unwrap();
        let mut buffers = Vec::new();
        for i in 0..self.assoc.len() {
            if let Some(ref x) = self.assoc[i] {
                buffers.push(x.get_handle().lock().unwrap().clone())
            } else if let BindType::Private(size, ty) = self.bindings[i].1 {
                match ty {
                    DataType::U32 => buffers.push(ResourceType::VU32(
                        CpuAccessibleBuffer::from_iter(
                            self.device.clone(),
                            BufferUsage::all(),
                            (0..size).map(|_| 0_u32),
                        ).unwrap(),
                    )),
                    DataType::I32 => buffers.push(ResourceType::VI32(
                        CpuAccessibleBuffer::from_iter(
                            self.device.clone(),
                            BufferUsage::all(),
                            (0..size).map(|_| 0_i32),
                        ).unwrap(),
                    )),
                    DataType::F32 => buffers.push(ResourceType::VF32(
                        CpuAccessibleBuffer::from_iter(
                            self.device.clone(),
                            BufferUsage::all(),
                            (0..size).map(|_| 0_f32),
                        ).unwrap(),
                    )),
                    DataType::Bool => unreachable!(),
                };
            } else {
                unreachable!();
            }
        }
        let writer = buffers.iter().enumerate().map(|(i, y)| match y {
            ResourceType::U32(x) => match self.version {
                VkVersion::Vulkan10 => unsafe { DescriptorWrite::uniform_buffer(i as u32, 0, x) },
                VkVersion::Vulkan11 => unsafe { DescriptorWrite::storage_buffer(i as u32, 0, x) },
            },
            ResourceType::I32(x) => match self.version {
                VkVersion::Vulkan10 => unsafe { DescriptorWrite::uniform_buffer(i as u32, 0, x) },
                VkVersion::Vulkan11 => unsafe { DescriptorWrite::storage_buffer(i as u32, 0, x) },
            },
            ResourceType::F32(x) => match self.version {
                VkVersion::Vulkan10 => unsafe { DescriptorWrite::uniform_buffer(i as u32, 0, x) },
                VkVersion::Vulkan11 => unsafe { DescriptorWrite::storage_buffer(i as u32, 0, x) },
            },
            ResourceType::VU32(x) => match self.version {
                VkVersion::Vulkan10 => unsafe { DescriptorWrite::uniform_buffer(i as u32, 0, x) },
                VkVersion::Vulkan11 => unsafe { DescriptorWrite::storage_buffer(i as u32, 0, x) },
            },
            ResourceType::VI32(x) => match self.version {
                VkVersion::Vulkan10 => unsafe { DescriptorWrite::uniform_buffer(i as u32, 0, x) },
                VkVersion::Vulkan11 => unsafe { DescriptorWrite::storage_buffer(i as u32, 0, x) },
            },
            ResourceType::VF32(x) => match self.version {
                VkVersion::Vulkan10 => unsafe { DescriptorWrite::uniform_buffer(i as u32, 0, x) },
                VkVersion::Vulkan11 => unsafe { DescriptorWrite::storage_buffer(i as u32, 0, x) },
            },
            ResourceType::Empty => unreachable!(),
        });
        unsafe { set.write(&self.device, writer) };
        let sanitized_set = MyDescriptorSet {
            set: set,
            layout: self.layout.clone(),
            buffers: buffers
        };
        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        ).unwrap()
            .dispatch([896, 1, 1], self.pipeline.clone(), sanitized_set, ())
            .unwrap()
            .build()
            .unwrap();
        let future = now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer).unwrap()
            .then_signal_fence_and_flush().unwrap();
        future.wait(None).unwrap();
        unsafe { self.pool.reset() }.unwrap();
        Ok("".into())
    }
}

struct MyDescriptorSet {
    set: UnsafeDescriptorSet,
    layout: ModuleLayout,
    buffers: Vec<ResourceType>
}

unsafe impl DescriptorSetDesc for MyDescriptorSet {
    fn num_bindings(&self) -> usize {
        self.layout.num_bindings_in_set(0).unwrap()
    }

    fn descriptor(&self, binding: usize) -> Option<DescriptorDesc> {
        eprintln!("Binding: {:?}", binding);
        self.layout.descriptor(0, binding)
    }
}

unsafe impl DescriptorSet for MyDescriptorSet {
    fn inner(&self) -> &UnsafeDescriptorSet {
        &self.set
    }

    fn num_buffers(&self) -> usize {
        self.buffers.len()
    }

    fn buffer(&self, index: usize) -> Option<(&BufferAccess, u32)> {
        if index >= self.buffers.len() {
            return None;
        }
        eprintln!("Index: {:?}", index);
        let desc_index = index as u32;
        match self.buffers[index] {
            ResourceType::U32(ref x) => Some((x, desc_index)),
            ResourceType::I32(ref x) => Some((x, desc_index)),
            ResourceType::F32(ref x) => Some((x, desc_index)),
            ResourceType::VU32(ref x) => Some((x, desc_index)),
            ResourceType::VI32(ref x) => Some((x, desc_index)),
            ResourceType::VF32(ref x) => Some((x, desc_index)),
            ResourceType::Empty => unreachable!(),
        }
    }

    fn num_images(&self) -> usize {
        0
    }

    fn image(&self, _index: usize) -> Option<(&ImageViewAccess, u32)> {
        None
    }
}
