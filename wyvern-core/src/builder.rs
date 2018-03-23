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

use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{spawn, JoinHandle};
use program::{Op, Program, Token, TokenId, TokenType};
use rand::{thread_rng, Rng};
use std::cmp::{Eq, PartialEq};

#[derive(Debug)]
pub struct ProgramBuilder {
    id: u64,
    worker_queue: SyncSender<WorkerMessage>,
    worker_thread: JoinHandle<Program>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProgramObjectInfo<'a> {
    pub(crate) token: Token,
    pub(crate) builder: &'a ProgramBuilder,
}

#[derive(Debug)]
enum WorkerMessage {
    Finalize,
    GenerateToken(SyncSender<TokenId>, TokenType),
    PushBlock,
    PopBlock(SyncSender<Vec<Op>>),
    AddOperation(Op),
}

impl<'a> ProgramBuilder {
    pub fn new() -> ProgramBuilder {
        let (tx, rx) = mpsc::sync_channel(0);
        ProgramBuilder {
            id: thread_rng().gen(),
            worker_queue: tx,
            worker_thread: spawn(move || worker(rx)),
        }
    }

    pub fn finalize(self) -> Program {
        self.worker_queue.send(WorkerMessage::Finalize).unwrap();
        self.worker_thread.join().unwrap()
    }

    pub(crate) fn gen_token(&'a self, ty: TokenType) -> ProgramObjectInfo<'a> {
        let (tx, rx) = mpsc::sync_channel(0);
        self.worker_queue
            .send(WorkerMessage::GenerateToken(tx, ty))
            .unwrap();
        ProgramObjectInfo {
            token: Token {
                id: rx.recv().unwrap(),
                ty,
            },
            builder: self,
        }
    }

    pub(crate) fn add_operation(&self, op: Op) {
        self.worker_queue
            .send(WorkerMessage::AddOperation(op))
            .unwrap();
    }
}

impl Default for ProgramBuilder {
    fn default() -> ProgramBuilder {
        ProgramBuilder::new()
    }
}

impl PartialEq for ProgramBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ProgramBuilder {}

fn worker(queue: Receiver<WorkerMessage>) -> Program {
    let queue = queue;
    let mut token_id = TokenId::default();
    let mut prog = Program::default();
    let mut block_stack: Vec<Vec<Op>> = Vec::new();
    let mut block_stack_top = Vec::new();
    loop {
        let message = queue.recv().unwrap();
        match message {
            WorkerMessage::Finalize => {
                assert_eq!(block_stack.len(), 0);
                prog.operations = block_stack_top;
                break prog;
            }
            WorkerMessage::GenerateToken(tx, ty) => {
                let id = token_id.next();
                let insert_result = prog.symbols.insert(id, ty);
                assert!(insert_result.is_none());
                tx.send(id).unwrap();
            }
            WorkerMessage::PushBlock => {
                block_stack.push(block_stack_top);
                block_stack_top = Vec::new();
            }
            WorkerMessage::PopBlock(tx) => {
                tx.send(block_stack_top).unwrap();
                assert!(!block_stack.is_empty());
                block_stack_top = block_stack.pop().unwrap();
            }
            WorkerMessage::AddOperation(op) => {
                block_stack_top.push(op);
            }
        }
    }
}
