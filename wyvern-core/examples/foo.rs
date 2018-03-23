#![feature(nll)]

extern crate wyvern_core;

use wyvern_core::builder::ProgramBuilder;
use wyvern_core::types::Constant;

fn main() {
    let builder = ProgramBuilder::new();
    let a = Constant::new(21, &builder);
    let _b = a * 2;
    println!("{:?}", builder.finalize());
}
