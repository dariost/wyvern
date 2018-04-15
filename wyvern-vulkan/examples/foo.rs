#![feature(nll, fs_read_write)]

extern crate byteorder;
extern crate wyvern_core as wcore;
extern crate wyvern_vulkan as wvulkan;

use byteorder::{ByteOrder, LittleEndian};
use std::fs::write;
use wcore::builder::ProgramBuilder;
use wcore::program::Program;
use wcore::types::Constant;
use wvulkan::generator::generate;

fn program() -> Program {
    let builder = ProgramBuilder::new();
    let a = builder.worker_id();
    let _b: Constant<i32> = Constant::from(a);
    let p1 = Constant::new(true, &builder);
    let p2 = Constant::new(true, &builder);
    let _p3 = p1 ^ p2;
    builder.finalize().unwrap()
}

fn u32tou8(v: &[u32]) -> Vec<u8> {
    let mut result = Vec::new();
    for i in v {
        let mut buf = [0; 4];
        LittleEndian::write_u32(&mut buf, *i);
        for j in &buf {
            result.push(*j);
        }
    }
    result
}

fn main() {
    let p = program();
    println!("Program: {:?}", p);
    write("foo.spv", u32tou8(&generate(&p).unwrap().0)).unwrap();
}
