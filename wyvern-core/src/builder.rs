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

use std::sync::mpsc;
use std::thread::{spawn, JoinHandle};
use program::Program;

#[derive(Debug)]
pub struct ProgramBuilder {
    worker_queue: mpsc::SyncSender<WorkerMessage>,
    worker_thread: JoinHandle<Program>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProgramObjectInfo<'a> {
    id: u64,
    builder: &'a ProgramBuilder,
}

#[derive(Debug)]
enum WorkerMessage {
    Finalize,
    GenerateId(mpsc::SyncSender<u64>),
}

impl<'a> ProgramBuilder {
    pub fn new() -> ProgramBuilder {
        let (tx, rx) = mpsc::sync_channel(0);
        ProgramBuilder {
            worker_queue: tx,
            worker_thread: spawn(move || worker(rx)),
        }
    }

    pub fn finalize(self) -> Program {
        self.worker_queue.send(WorkerMessage::Finalize).unwrap();
        self.worker_thread.join().unwrap()
    }

    pub(crate) fn gen_poi(&'a self) -> ProgramObjectInfo<'a> {
        let (tx, rx) = mpsc::sync_channel(0);
        self.worker_queue
            .send(WorkerMessage::GenerateId(tx))
            .unwrap();
        ProgramObjectInfo {
            id: rx.recv().unwrap(),
            builder: self,
        }
    }
}

impl Default for ProgramBuilder {
    fn default() -> ProgramBuilder {
        ProgramBuilder::new()
    }
}

fn worker(queue: mpsc::Receiver<WorkerMessage>) -> Program {
    let mut next_id = 0;
    loop {
        let message = queue.recv().unwrap();
        match message {
            WorkerMessage::Finalize => break Program {},
            WorkerMessage::GenerateId(tx) => {
                tx.send(next_id).unwrap();
                next_id += 1;
            }
            _ => continue,
        }
    }
}
