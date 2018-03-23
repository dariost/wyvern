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

use wcore::executor::{Executable, IO};
use std::sync::Arc;
use resource::CpuResource;

#[derive(Debug)]
pub struct CpuExecutable {}

impl Executable for CpuExecutable {
    type Error = String;
    type Report = String;
    type Resource = CpuResource;

    fn bind<S: ToString>(&mut self, name: S, kind: IO, resource: Arc<CpuResource>) {}

    fn unbind<S: ToString>(&mut self, name: S, kind: IO) {}

    fn run(&mut self) -> Result<String, String> {
        Ok(String::new())
    }
}
