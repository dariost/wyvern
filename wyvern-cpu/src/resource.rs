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
use std::sync::Mutex;
use wcore::executor::Resource;
use wcore::program::{get_token_type, TokenType, TokenValue};

#[derive(Debug)]
pub struct CpuResource {
    pub(crate) id: u64,
    pub(crate) data: Mutex<TokenValue>,
}

impl Resource for CpuResource {
    fn clear(&self) {
        *self.data.lock().unwrap() = TokenValue::Null;
    }

    fn token_type(&self) -> TokenType {
        get_token_type(&self.data.lock().unwrap())
    }

    fn get_data(&self) -> TokenValue {
        self.data.lock().unwrap().clone()
    }

    fn set_data(&self, value: TokenValue) {
        *self.data.lock().unwrap() = value;
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
