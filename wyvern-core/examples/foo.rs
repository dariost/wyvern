#![feature(nll)]

extern crate wyvern_core;

use wyvern_core::builder::ProgramBuilder;
use wyvern_core::types::{Array, Constant, Variable};

fn main() {
    let builder = ProgramBuilder::new();
    let _100 = Constant::new(100, &builder);
    let _20 = Constant::new(20, &builder);
    let v: Array<u32> = Array::new(_100, 120, &builder);
    builder.if_then(
        |_| _20.lt(_100),
        |ctx| ctx.while_loop(|_| _20.lt(_100), |_| v.at(_20).store(_100)),
    );
    let _a = v.at(_20).load();
    let program = builder.finalize().unwrap();
    println!("{:?}", program);
}
