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

use executable::VkExecutable;
use generator::{generate, Binding, VkVersion};
use rand::{thread_rng, Rng};
use resource::ResourceType;
use resource::VkResource;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use vulkano::descriptor::descriptor::DescriptorDesc;
use vulkano::descriptor::descriptor::{DescriptorBufferDesc, DescriptorDescTy, ShaderStages};
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::descriptor::pipeline_layout::{PipelineLayoutDesc, PipelineLayoutDescPcRange};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice, Version};
use vulkano::pipeline::shader::ShaderModule;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::ComputePipelineAbstract;
use wcore::executor::Executor;
use wcore::program::Program;

#[derive(Debug)]
pub struct VkExecutor {
    instance: Arc<Instance>,
    queue: Arc<Queue>,
    device: Arc<Device>,
    version: VkVersion,
}

impl Executor for VkExecutor {
    type Config = ();
    type Error = String;
    type Resource = VkResource;
    type Executable = VkExecutable;

    fn new(_config: ()) -> Result<VkExecutor, String> {
        let instance =
            Instance::new(None, &InstanceExtensions::none(), None).map_err(|x| format!("{:?}", x))?;
        let physical_device = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or("No Vulkan device found")?;
        let version = match physical_device.api_version() {
            Version {
                major: 1, minor: 0, ..
            } => VkVersion::Vulkan10,
            Version {
                major: 1, minor: 1, ..
            } => VkVersion::Vulkan11,
            _ => VkVersion::Vulkan11,
        };
        let queue = physical_device
            .queue_families()
            .find(|&q| q.supports_compute())
            .ok_or("No compute queue found")?;
        let (device, mut queues) = {
            Device::new(
                physical_device,
                physical_device.supported_features(),
                &DeviceExtensions::none(),
                [(queue, 0.5)].iter().cloned(),
            ).map_err(|x| format!("{:?}", x))?
        };
        let queue = queues.next().ok_or("No queue found")?;
        Ok(VkExecutor {
            instance,
            queue,
            device,
            version,
        })
    }

    fn compile(&self, program: Program) -> Result<VkExecutable, String> {
        let (binary, bindings) = generate(&program, self.version)?;
        // TODO: validate and optimize the binary
        let module = unsafe {
            ShaderModule::from_words(self.device.clone(), &binary).map_err(|x| format!("{:?}", x))?
        };
        let num_bindings = bindings.iter().map(|x| x.0 + 1).max().unwrap_or(0);
        let layout = ModuleLayout {
            bindings: bindings.clone(),
            num_bindings,
        };
        let pipeline = Arc::new({
            ComputePipeline::new(
                self.device.clone(),
                &unsafe { module.compute_entry_point(&CString::new("main").unwrap(), layout) },
                &(),
            ).map_err(|x| format!("{:?}", x))?
        });
        let pool: FixedSizeDescriptorSetsPool<Arc<ComputePipelineAbstract>> =
            FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);
        Ok(VkExecutable {
            pool,
            module,
            bindings,
            program,
            assoc: vec![None; num_bindings as usize],
        })
    }

    fn new_resource(&self) -> Result<Arc<VkResource>, String> {
        Ok(Arc::new(VkResource {
            id: thread_rng().gen(),
            resource: Arc::new(Mutex::new(ResourceType::Empty)),
            device: self.device.clone(),
        }))
    }
}

#[derive(Clone)]
struct ModuleLayout {
    bindings: Vec<Binding>,
    num_bindings: u32,
}

unsafe impl PipelineLayoutDesc for ModuleLayout {
    fn num_sets(&self) -> usize {
        1
    }

    fn num_bindings_in_set(&self, set: usize) -> Option<usize> {
        if set > 0 {
            None
        } else {
            Some(self.num_bindings as usize)
        }
    }

    fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
        if set > 0 || binding >= self.num_bindings as usize {
            return None;
        }
        for bind in &self.bindings {
            if bind.0 as usize == binding {
                return Some(DescriptorDesc {
                    array_count: 1,
                    stages: ShaderStages::compute(),
                    readonly: false,
                    ty: DescriptorDescTy::Buffer(DescriptorBufferDesc {
                        storage: true,
                        dynamic: Some(bind.2),
                    }),
                });
            }
        }
        unreachable!();
    }

    fn num_push_constants_ranges(&self) -> usize {
        0
    }

    fn push_constants_range(&self, _num: usize) -> Option<PipelineLayoutDescPcRange> {
        None
    }
}
