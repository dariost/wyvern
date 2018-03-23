#![feature(nll)]

extern crate wyvern_core;

use wyvern_core::builder::ProgramBuilder;
use wyvern_core::types::{Constant, Variable};

fn main() {
    let builder = ProgramBuilder::new();
    let a = Constant::new(41, &builder);
    let b = !a;
    let program = builder.finalize().unwrap();
    println!("{:?}", program);
}
