#!/usr/bin/env python3

import sys
import os

sys.path.append(os.path.abspath("./"))
sys.path.append(os.path.abspath("../"))

import wyvern as wy


def add(N):
    builder = wy.ProgramBuilder()
    ctx = builder.newContext()
    ctx.declVariable("n", wy.DataType.uint32, wy.IoType.input)
    ctx.declArray("a", wy.DataType.uint32, wy.IoType.input,
                  ctx.n, N)
    ctx.declArray("b", wy.DataType.uint32, wy.IoType.input,
                  ctx.n, N)
    ctx.declArray("c", wy.DataType.uint32, wy.IoType.output,
                  ctx.n, N)
    ctx.tid = ctx.workerId()
    ctx.tsize = ctx.numWorkers()
    def loop():
        ctx.c[ctx.tid] = ctx.a[ctx.tid] + ctx.b[ctx.tid]
        ctx.tid = ctx.tid + ctx.tsize
    ctx.While(lambda: ctx.tid < ctx.n, loop)
    return builder.finalize()


if __name__ == "__main__":
    N = 1 << 24
    program = add(N)
    executor = wy.WyVkExecutor()
    executable = executor.compile(program)
    a = executor.newResource()
    b = executor.newResource()
    c = executor.newResource()
    n = executor.newResource()
    a.set_data_array_uint32(list(range(N)))
    b.set_data_array_uint32(list(range(N, 0, -1)))
    c.set_data_array_uint32([0] * N)
    n.set_data_uint32(N)
    executable.bind("a", wy.IoType.input.value, a)
    executable.bind("b", wy.IoType.input.value, b)
    executable.bind("c", wy.IoType.output.value, c)
    executable.bind("n", wy.IoType.input.value, n)
    executable.run()
    result = c.get_data_array_uint32()
    for i in result:
        assert i == N
