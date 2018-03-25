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

use executable::CpuExecutable;
use rand::{thread_rng, Rng};
use resource::CpuResource;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wcore::executor::Executor;
use wcore::program::{Program, TokenValue};

#[derive(Debug)]
pub struct CpuExecutor {}

impl Executor for CpuExecutor {
    type Config = ();
    type Error = String;
    type Resource = CpuResource;
    type Executable = CpuExecutable;

    fn new(_config: ()) -> Result<CpuExecutor, String> {
        Ok(CpuExecutor {})
    }

    fn compile(&self, program: Program) -> Result<CpuExecutable, String> {
        Ok(CpuExecutable {
            program,
            binding: HashMap::new(),
        })
    }

    fn new_resource(&self) -> Result<Arc<CpuResource>, String> {
        Ok(Arc::new(CpuResource {
            id: thread_rng().next_u64(),
            data: Mutex::new(TokenValue::Null),
        }))
    }
}
