#![feature(nll, fs_read_write)]

extern crate byteorder;
extern crate wyvern_core as wcore;
extern crate wyvern_vulkan as wvulkan;

use byteorder::{ByteOrder, LittleEndian};
use std::fs::write;
use wcore::builder::ProgramBuilder;
use wcore::types::Constant;
use wvulkan::generator::generate;

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
    let builder = ProgramBuilder::new();
    program(&builder);
    let p = builder.finalize().unwrap();
    write("foo.spv", u32tou8(&generate(&p).unwrap().0)).unwrap();
}

fn program(builder: &ProgramBuilder) {
    let a = Constant::new(42.0, &builder);
    let b = Constant::new(21.2, &builder);
    builder.while_loop(
        |_| (a / 2.0).lt(b),
        |_| {
            builder.if_then(
                |_| b.gt(a + 3.0),
                |_| {
                    let _c = a + b + Constant::from(builder.num_workers());
                },
            )
        },
    );
}
