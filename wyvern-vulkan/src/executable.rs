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

use generator::{BindType, Binding};
use resource::ResourceType;
use resource::VkResource;
use std::collections::HashMap;
use std::mem::swap;
use std::sync::Arc;
use std::sync::Mutex;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::pipeline::shader::ShaderModule;
use vulkano::pipeline::ComputePipelineAbstract;
use wcore::executor::{Executable, IO};
use wcore::program::Program;

pub struct VkExecutable {
    pub(crate) module: Arc<ShaderModule>,
    pub(crate) program: Program,
    pub(crate) bindings: Vec<Binding>,
    pub(crate) assoc: Vec<Option<Arc<VkResource>>>,
    pub(crate) pool: FixedSizeDescriptorSetsPool<Arc<ComputePipelineAbstract>>,
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
        unimplemented!();
    }
}
