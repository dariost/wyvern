#![allow(unused_imports)]

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
    let in1 = executor.new_resource().unwrap();
    let in2 = executor.new_resource().unwrap();
    let out = executor.new_resource().unwrap();
    in1.set_data(TokenValue::Vector(ConstantVector::U32(vec![2; 10000])));
    in2.set_data(TokenValue::Scalar(ConstantScalar::U32(3)));
    out.set_data(TokenValue::Scalar(ConstantScalar::U32(0)));
    executable.bind("out", IO::Output, out.clone());
    executable.bind("in1", IO::Input, in1.clone());
    executable.bind("in2", IO::Input, in2.clone());
    executable.run().unwrap();
    println!("{:?}", out.get_data());
}

fn program(builder: &ProgramBuilder) {
    let zero = Constant::new(0_u32, builder);
    let one = Constant::new(1_u32, builder);
    let v: Array<u32> = Array::new(zero, 10000, true, builder).mark_as_input("in1");
    let a: Constant<u32> = Variable::new(builder).mark_as_input("in2").load();
    let o = Variable::new(builder).mark_as_output("out");
    let id = builder.worker_id();
    let size = builder.num_workers();
    let tmp: Array<u32> = Array::new(size, 10000, true, builder);
    let v_length = v.len();
    let index = Variable::new(builder);
    index.store(id);
    builder.while_loop(|_| {
        index.load().lt(v_length)
    }, |_| {
        let old_value = tmp.at(id).load();
        tmp.at(id).store(old_value + v.at(index.load()).load() * a);
        index.store(index.load() + size);
    });
    builder.memory_barrier();
    builder.if_then(|_| {
        id.eq(zero)
    }, |_| {
        index.store(zero);
        builder.while_loop(|_| {
            index.load().lt(size)
        }, |_| {
            o.store(o.load() + tmp.at(index.load()).load());
            index.store(index.load() + one);
        });
    });
}
