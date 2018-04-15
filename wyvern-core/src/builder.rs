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

use program::{LabelId, Op, Program, Token, TokenId, TokenType};
use rand::{thread_rng, Rng};
use std::cmp::{Eq, PartialEq};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{spawn, JoinHandle};
use types::Constant;

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
pub(crate) enum WorkerMessage {
    Finalize,
    GenerateToken(SyncSender<TokenId>, TokenType),
    PushBlock,
    PopBlock(SyncSender<Vec<Op>>),
    AddOperation(Op),
    MarkInput(TokenId, String),
    MarkOutput(TokenId, String),
    NextLabel(SyncSender<LabelId>),
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

    pub fn finalize(self) -> Result<Program, String> {
        self.send_message(WorkerMessage::Finalize);
        Ok(self.worker_thread.join().unwrap())
    }

    pub fn memory_barrier(&self) {
        self.add_operation(Op::MemoryBarrier);
    }

    pub fn barrier(&self) {
        self.add_operation(Op::ControlBarrier);
    }

    pub fn worker_id(&'a self) -> Constant<'a, u32> {
        let result = Constant::generate(self);
        self.add_operation(Op::WorkerId(result.info.token.id));
        result
    }

    pub fn if_then<T: Fn(&ProgramBuilder) -> Constant<'a, bool>, U: Fn(&ProgramBuilder) -> ()>(
        &self,
        cond: T,
        body: U,
    ) {
        self.send_message(WorkerMessage::PushBlock);
        let condition = cond(self);
        let (tx, rx) = mpsc::sync_channel(0);
        self.send_message(WorkerMessage::PopBlock(tx));
        let cond = rx.recv().unwrap();
        self.send_message(WorkerMessage::PushBlock);
        body(self);
        let (tx, rx) = mpsc::sync_channel(0);
        self.send_message(WorkerMessage::PopBlock(tx));
        let body = rx.recv().unwrap();
        self.add_operation(Op::If(
            cond,
            condition.info.token.id,
            self.next_label(),
            body,
            self.next_label(),
        ));
    }

    pub fn if_then_else<
        T: Fn(&ProgramBuilder) -> Constant<'a, bool>,
        U: Fn(&ProgramBuilder) -> (),
    >(
        &self,
        cond: T,
        body_if: U,
        body_else: U,
    ) {
        self.send_message(WorkerMessage::PushBlock);
        let condition = cond(self);
        let (tx, rx) = mpsc::sync_channel(0);
        self.send_message(WorkerMessage::PopBlock(tx));
        let cond = rx.recv().unwrap();
        self.send_message(WorkerMessage::PushBlock);
        body_if(self);
        let (tx, rx) = mpsc::sync_channel(0);
        self.send_message(WorkerMessage::PopBlock(tx));
        let body_if = rx.recv().unwrap();
        self.send_message(WorkerMessage::PushBlock);
        body_else(self);
        let (tx, rx) = mpsc::sync_channel(0);
        self.send_message(WorkerMessage::PopBlock(tx));
        let body_else = rx.recv().unwrap();
        self.add_operation(Op::IfElse(
            cond,
            condition.info.token.id,
            self.next_label(),
            body_if,
            self.next_label(),
            body_else,
            self.next_label(),
        ));
    }

    pub fn while_loop<
        T: Fn(&ProgramBuilder) -> Constant<'a, bool>,
        U: Fn(&ProgramBuilder) -> (),
    >(
        &self,
        cond: T,
        body: U,
    ) {
        self.send_message(WorkerMessage::PushBlock);
        let condition = cond(self);
        let (tx, rx) = mpsc::sync_channel(0);
        self.send_message(WorkerMessage::PopBlock(tx));
        let cond = rx.recv().unwrap();
        self.send_message(WorkerMessage::PushBlock);
        body(self);
        let (tx, rx) = mpsc::sync_channel(0);
        self.send_message(WorkerMessage::PopBlock(tx));
        let body = rx.recv().unwrap();
        self.add_operation(Op::While(
            self.next_label(),
            cond,
            condition.info.token.id,
            self.next_label(),
            body,
            self.next_label(),
        ));
    }

    pub fn num_workers(&'a self) -> Constant<'a, u32> {
        let result = Constant::generate(self);
        self.add_operation(Op::NumWorkers(result.info.token.id));
        result
    }

    pub(crate) fn gen_token(&'a self, ty: TokenType) -> ProgramObjectInfo<'a> {
        let (tx, rx) = mpsc::sync_channel(0);
        self.send_message(WorkerMessage::GenerateToken(tx, ty));
        ProgramObjectInfo {
            token: Token {
                id: rx.recv().unwrap(),
                ty,
            },
            builder: self,
        }
    }

    pub(crate) fn add_operation(&self, op: Op) {
        self.send_message(WorkerMessage::AddOperation(op));
    }

    pub(crate) fn send_message(&self, msg: WorkerMessage) {
        self.worker_queue.send(msg).unwrap();
    }

    pub(crate) fn next_label(&self) -> LabelId {
        let (tx, rx) = mpsc::sync_channel(0);
        self.send_message(WorkerMessage::NextLabel(tx));
        rx.recv().unwrap()
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
    let mut label_id = LabelId::default();
    label_id.next();
    let mut token_id = TokenId::default();
    let mut prog = Program::default();
    let mut block_stack: Vec<Vec<Op>> = Vec::new();
    let mut block_stack_top = Vec::new();
    loop {
        let message = queue.recv().unwrap();
        match message {
            WorkerMessage::Finalize => {
                assert_eq!(block_stack.len(), 0);
                prog.operation = block_stack_top;
                break prog;
            }
            WorkerMessage::GenerateToken(tx, ty) => {
                let id = token_id.next();
                let insert_result = prog.symbol.insert(id, ty);
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
            WorkerMessage::MarkInput(id, name) => {
                prog.input.insert(name, id);
            }
            WorkerMessage::MarkOutput(id, name) => {
                prog.output.insert(name, id);
            }
            WorkerMessage::NextLabel(tx) => {
                tx.send(label_id.next()).unwrap();
            }
        }
    }
}
