#![feature(nll, fs_read_write)]

extern crate byteorder;
extern crate wyvern_core as wcore;
extern crate wyvern_vulkan as wvulkan;

use byteorder::{ByteOrder, LittleEndian};
use std::fs::write;
use wcore::builder::ProgramBuilder;
use wcore::types::{Array, Constant, Variable};
use wvulkan::generator::{generate, Version};

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
    bfs(&builder, 100, 1000);
    let p = builder.finalize().unwrap();
    write(
        "bfs.spv",
        u32tou8(&generate(&p, Version::Vulkan11).unwrap().0),
    ).unwrap();
}

fn bfs(builder: &ProgramBuilder, v: u32, e: u32) {
    let zero = Constant::new(0_u32, builder);
    let one = Constant::new(1_u32, builder);
    let nodes: Array<u32> = Array::new(zero, v + 1, true, builder).mark_as_input("nodes");
    let edges: Array<u32> = Array::new(zero, e, true, builder).mark_as_input("edges");
    let dist: Array<u32> = Array::new(zero, v, true, builder).mark_as_output("distance");
    let stop: Array<u32> = Array::new(zero, v, true, builder);
    let starting_node: Constant<u32> = Variable::new(builder).mark_as_input("starting_node").load();
    let id = builder.worker_id();
    let size = builder.num_workers();
    let num_nodes = nodes.len() - 1;
    let node = Variable::new(builder);
    node.store(id);
    builder.while_loop(
        |_| node.load().lt(num_nodes),
        |_| {
            let nid = node.load();
            dist.at(nid).store(zero);
            node.store(nid + size);
        },
    );
    builder.memory_barrier();
    let iteration = Variable::new(builder);
    iteration.store(zero);
    stop.at(zero).store(zero);
    builder.while_loop(
        |_| stop.at(zero).load().ne(one),
        |_| {
            stop.at(zero).store(one);
            builder.while_loop(
                |_| node.load().lt(num_nodes),
                |_| {
                    let nid = node.load();
                    builder.if_then(
                        |_| {
                            dist.at(nid).load().eq(iteration.load())
                                & (dist.at(nid).load().ne(zero) | nid.eq(starting_node))
                        },
                        |_| {
                            let adj_idx = Variable::new(builder);
                            adj_idx.store(nodes.at(nid).load());
                            builder.while_loop(
                                |_| adj_idx.load().lt(nodes.at(nid + 1).load()),
                                |_| {
                                    let adj = edges.at(adj_idx.load()).load();
                                    builder.if_then(
                                        |_| dist.at(adj).load().eq(zero) & adj.ne(starting_node),
                                        |_| {
                                            dist.at(adj).store(iteration.load() + 1);
                                        },
                                    );
                                    adj_idx.store(adj_idx.load() + 1);
                                },
                            );
                        },
                    );
                    node.store(nid + size);
                },
            );
            builder.memory_barrier();
            builder.while_loop(
                |_| node.load().lt(num_nodes),
                |_| {
                    let nid = node.load();
                    builder.if_then(
                        |_| dist.at(nid).load().eq(zero) & nid.ne(starting_node),
                        |_| {
                            stop.at(zero).store(zero);
                        },
                    );
                    node.store(nid + size);
                },
            );
            builder.memory_barrier();
            iteration.store(iteration.load() + 1);
        },
    );
}
