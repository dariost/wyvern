#![feature(nll)]

extern crate wyvern_core as wcore;

use wcore::builder::ProgramBuilder;
use wcore::program::Program;
use wcore::types::{Constant, Variable, Array};

fn add(n: u32) -> Program {
    let builder = ProgramBuilder::new();
    let z: Constant<u32> = Constant::new(0, &builder);
    let a: Array<u32> = Array::new(z, n, true, &builder).mark_as_input("a");
    let b: Array<u32> = Array::new(z, n, true, &builder).mark_as_input("b");
    let c: Array<u32> = Array::new(z, n, true, &builder).mark_as_output("c");
    let n = Constant::new(n, &builder);
    let tid = Variable::new(&builder);
    tid.store(builder.worker_id());
    builder.while_loop(|_| tid.load().lt(n), |_| {
        let i = tid.load();
        c.at(i).store(a.at(i).load() + b.at(i).load());
        tid.store(i + builder.num_workers());
    });
    builder.finalize().unwrap()
}

fn main() {
    println!("{:?}", add(1 << 24));
}
