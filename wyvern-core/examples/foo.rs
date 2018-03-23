#![feature(nll)]

extern crate wyvern_core;

use wyvern_core::builder::ProgramBuilder;
use wyvern_core::types::Variable;

fn main() {
    let builder = ProgramBuilder::new();
    Variable::new(&builder)
        .mark_as_output("out")
        .store(Variable::<f32>::new(&builder).mark_as_input("in").load() * 2.0);
    let program = builder.finalize().unwrap();
    println!("{:?}", program);
}
