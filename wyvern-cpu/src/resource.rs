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

use std::hash::{Hash, Hasher};
use wcore::program::{get_token_type, TokenType, TokenValue};
use wcore::executor::Resource;

#[derive(Debug)]
pub struct CpuResource {
    id: u64,
    data: TokenValue,
}

impl Resource for CpuResource {
    fn clear(&mut self) {
        self.data = TokenValue::Null;
    }

    fn token_type(&self) -> TokenType {
        get_token_type(&self.data)
    }

    fn get_data(&self) -> TokenValue {
        self.data.clone()
    }

    fn set_data(&mut self, value: TokenValue) {
        self.data = value;
    }
}

impl PartialEq for CpuResource {
    fn eq(&self, other: &CpuResource) -> bool {
        self.id == other.id
    }
}

impl Eq for CpuResource {}

impl Hash for CpuResource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
