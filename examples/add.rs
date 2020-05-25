extern crate wyvern;
use std::sync::Arc;
use wyvern::core as wcore;

use wcore::builder::ProgramBuilder;
use wcore::program::Program;
use wcore::types::{Array, Constant, Variable};
use wyvern::core::executor::{Executable, Executor, Resource, IO};
use wyvern::core::program::{ConstantVector, TokenValue};
use wyvern::vk::executor::VkExecutor;

fn add(n: u32) -> Program {
    let builder = ProgramBuilder::new();
    let z: Constant<u32> = Constant::new(0, &builder);
    let a: Array<u32> = Array::new(z, n, true, &builder).mark_as_input("a");
    let b: Array<u32> = Array::new(z, n, true, &builder).mark_as_input("b");
    let c: Array<u32> = Array::new(z, n, true, &builder).mark_as_output("c");
    let n = Constant::new(n, &builder);
    let tid = Variable::new(&builder);
    tid.store(builder.worker_id());
    builder.while_loop(
        |_| tid.load().lt(n),
        |_| {
            let i = tid.load();
            c.at(i).store(a.at(i).load() + b.at(i).load());
            tid.store(i + builder.num_workers());
        },
    );
    builder.finalize().unwrap()
}

fn get_vec_u32<T: Resource>(a: Arc<T>) -> Vec<u32> {
    match a.get_data() {
        TokenValue::Vector(ConstantVector::U32(x)) => x,
        _ => unreachable!(),
    }
}

fn main() {
    const N: u32 = 1 << 24;
    let program = add(N);
    let executor = VkExecutor::new(Default::default()).unwrap();
    let mut executable = executor.compile(program).unwrap();
    let a = executor.new_resource().unwrap();
    let b = executor.new_resource().unwrap();
    let c = executor.new_resource().unwrap();
    a.set_data(TokenValue::Vector(ConstantVector::U32(
        (0..N).map(|x| x).collect(),
    )));
    b.set_data(TokenValue::Vector(ConstantVector::U32(
        (0..N).map(|x| N - x).collect(),
    )));
    c.set_data(TokenValue::Vector(ConstantVector::U32(vec![0; N as usize])));
    executable.bind("a", IO::Input, a.clone());
    executable.bind("b", IO::Input, b.clone());
    executable.bind("c", IO::Output, c.clone());
    executable.run().unwrap();
    let output = get_vec_u32(c.clone());
    for i in &output {
        assert_eq!(*i, N);
    }
}
