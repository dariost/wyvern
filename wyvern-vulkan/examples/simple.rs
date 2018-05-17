#![feature(nll)]

extern crate wyvern_core as wcore;
extern crate wyvern_vulkan as wvk;

use wcore::builder::ProgramBuilder;
use wcore::executor::{Executable, Executor, Resource, IO};
use wcore::program::{ConstantScalar, ConstantVector, TokenValue};
use wcore::types::{Array, Constant, Variable};
use wvk::executor::VkExecutor;

fn main() {
    let builder = ProgramBuilder::new();
    program(&builder);
    let program = builder.finalize().unwrap();
    let executor = VkExecutor::new(Default::default()).unwrap();
    let mut executable = executor.compile(program).unwrap();
    let asd = executor.new_resource().unwrap();
    let foo = executor.new_resource().unwrap();
    let bar = executor.new_resource().unwrap();
    foo.set_data(TokenValue::Scalar(ConstantScalar::U32(0)));
    asd.set_data(TokenValue::Scalar(ConstantScalar::U32(2)));
    bar.set_data(TokenValue::Scalar(ConstantScalar::U32(21)));
    executable.bind("foo", IO::Output, foo.clone());
    executable.bind("asd", IO::Input, asd.clone());
    executable.bind("bar", IO::Input, asd.clone());
    executable.run().unwrap();
    println!("{:?}", foo.get_data());
}

fn program(builder: &ProgramBuilder) {
    let x = Variable::new(builder).mark_as_input("asd").load();
    let y = Variable::new(builder).mark_as_input("bar").load();
    let a: Constant<u32> = y * x;
    let b = Variable::new(builder).mark_as_output("foo");
    b.store(a);
}
