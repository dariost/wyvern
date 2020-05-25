extern crate wyvern_core as wcore;
extern crate wyvern_cpu as wcpu;

use wcore::builder::ProgramBuilder;
use wcore::executor::{Executable, Executor, Resource, IO};
use wcore::program::{ConstantScalar, Program, TokenValue};
use wcore::types::{Constant, Variable};
use wcpu::executor::CpuExecutor;

fn prog() -> Program {
    let builder = ProgramBuilder::new();
    let a: Constant<u32> = Variable::new(&builder).mark_as_input("in").load();
    let mut r = a << 10;
    r = r | a;
    let r = Constant::from(r);
    let r = r / 2.0;
    let _b = Variable::new(&builder).mark_as_output("out").store(r);
    builder.finalize().unwrap()
}

fn main() {
    let program = prog();
    let executor = CpuExecutor::new(Default::default()).unwrap();
    let mut executable = executor.compile(program).unwrap();
    let input = executor.new_resource().unwrap();
    let output = executor.new_resource().unwrap();
    input.set_data(TokenValue::Scalar(ConstantScalar::U32(1)));
    executable.bind("in", IO::Input, input.clone());
    executable.bind("out", IO::Output, output.clone());
    executable.run().unwrap();
    let value = output.get_data();
    let value = match value {
        TokenValue::Scalar(ConstantScalar::F32(x)) => x,
        _ => unreachable!(),
    };
    println!("{}", value);
}
