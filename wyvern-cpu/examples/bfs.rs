#![feature(nll)]

extern crate wyvern_core as wcore;
extern crate wyvern_cpu as wcpu;

use std::io::stdin;
use wcore::builder::ProgramBuilder;
use wcore::executor::{Executable, Executor, Resource, IO};
use wcore::program::{ConstantScalar, ConstantVector, TokenValue};
use wcore::types::{Array, Constant, Variable};
use wcpu::executor::CpuExecutor;

fn read_pair() -> (usize, usize) {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    let v: Vec<_> = buffer
        .trim()
        .split(" ")
        .map(|x| x.parse().unwrap())
        .collect();
    (v[0], v[1])
}

fn main() {
    let (n, m) = read_pair();
    let mut children = vec![Vec::new(); n];
    for _ in 0..m {
        let (a, b) = read_pair();
        children[a].push(b);
        children[b].push(a);
    }
    let mut nodes = vec![0_u32; n + 1];
    let mut edges = vec![0_u32; m * 2];
    let dist = vec![0_u32; n];
    let mut index = 0;
    for i in 0..=n {
        nodes[i] = index as u32;
        if i == n {
            continue;
        }
        for adj in &children[i] {
            edges[index] = *adj as u32;
            index += 1;
        }
    }
    let builder = ProgramBuilder::new();
    bfs(&builder, n as u32, m as u32);
    let program = builder.finalize().unwrap();
    let executor = CpuExecutor::new(Default::default()).unwrap();
    let mut executable = executor.compile(program).unwrap();
    let nodes_resource = executor.new_resource().unwrap();
    let edges_resource = executor.new_resource().unwrap();
    let dist_resource = executor.new_resource().unwrap();
    let starting_node_resource = executor.new_resource().unwrap();
    nodes_resource.set_data(TokenValue::Vector(ConstantVector::U32(nodes)));
    edges_resource.set_data(TokenValue::Vector(ConstantVector::U32(edges)));
    dist_resource.set_data(TokenValue::Vector(ConstantVector::U32(dist)));
    executable.bind("nodes", IO::Input, nodes_resource.clone());
    executable.bind("edges", IO::Input, edges_resource.clone());
    executable.bind("starting_node", IO::Input, starting_node_resource.clone());
    executable.bind("distance", IO::Output, dist_resource.clone());
    let mut max_distance = 0;
    for i in 0..n {
        starting_node_resource.set_data(TokenValue::Scalar(ConstantScalar::U32(i as u32)));
        executable.run().unwrap();
        let value = match dist_resource.get_data() {
            TokenValue::Vector(ConstantVector::U32(x)) => x,
            _ => unreachable!(),
        };
        for v in value {
            if v > max_distance {
                max_distance = v;
            }
        }
        eprintln!("Processed {}/{}", i + 1, n);
    }
    println!("Diameter: {}", max_distance);
}

fn bfs(builder: &ProgramBuilder, v: u32, e: u32) {
    let zero = Constant::new(0_u32, builder);
    let one = Constant::new(1_u32, builder);
    let nodes: Array<u32> = Array::new(zero, v + 1, true, builder).mark_as_input("nodes");
    let edges: Array<u32> = Array::new(zero, e * 2, true, builder).mark_as_input("edges");
    let dist: Array<u32> = Array::new(zero, v, true, builder).mark_as_output("distance");
    let stop: Array<u32> = Array::new(one, 1, true, builder);
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
            node.store(id);
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
            node.store(id);
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
