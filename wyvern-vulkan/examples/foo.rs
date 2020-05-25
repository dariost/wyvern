extern crate byteorder;
extern crate wyvern_core as wcore;
extern crate wyvern_vulkan as wvulkan;

use byteorder::{ByteOrder, LittleEndian};
use std::fs::write;
use wcore::builder::ProgramBuilder;
use wcore::types::{Constant, Variable};
use wvulkan::generator::{generate, VkVersion};

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
    write("foo.spv", u32tou8(&generate(&p, VkVersion::Vulkan11).unwrap().0)).unwrap();
}

fn program(builder: &ProgramBuilder) {
    let a: Constant<f32> = Variable::new(builder).mark_as_input("lol").load();
    let b: Constant<f32> = Variable::new(builder).mark_as_output("lal").load();
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
