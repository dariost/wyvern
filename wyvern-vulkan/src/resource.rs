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

use wcore::executor::Resource;
use wcore::program::{TokenType, TokenValue};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct VkResource {}

impl Resource for VkResource {
    fn clear(&self) {
        unimplemented!();
    }

    fn token_type(&self) -> TokenType {
        unimplemented!();
    }

    fn set_data(&self, value: TokenValue) {
        unimplemented!();
    }

    fn get_data(&self) -> TokenValue {
        unimplemented!();
    }
}
