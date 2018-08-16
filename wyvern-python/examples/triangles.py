#!/usr/bin/env python3

import sys
import os

sys.path.append(os.path.abspath("./"))
sys.path.append(os.path.abspath("../"))

import wyvern as wy


def mul(n):
    builder = wy.ProgramBuilder()
    ctx = builder.newContext()
    ctx.declVariable("n", wy.DataType.uint32, wy.IoType.input)
    ctx.declArray("a", wy.DataType.uint32, wy.IoType.input,
                  ctx.n * ctx.n, n * n)
    ctx.declArray("b", wy.DataType.uint32, wy.IoType.input,
                  ctx.n * ctx.n, n * n)
    ctx.declArray("c", wy.DataType.uint32, wy.IoType.output,
                  ctx.n * ctx.n, n * n)
    ctx.tid = ctx.workerId()
    ctx.tsize = ctx.numWorkers()

    def loop():
        ctx.i = ctx.tid % ctx.n
        ctx.j = ctx.tid / ctx.n
        ctx.k = 0
        ctx.acc = 0

        def inner_loop():
            ctx.acc = ctx.acc + ctx.a[ctx.i * ctx.n + ctx.k] * \
                                ctx.b[ctx.k * ctx.n + ctx.j]
            ctx.k = ctx.k + 1

        ctx.While(lambda: ctx.k < ctx.n, inner_loop)
        ctx.c[ctx.i * ctx.n + ctx.j] = ctx.acc
        ctx.tid = ctx.tid + ctx.tsize

    ctx.While(lambda: ctx.tid < (ctx.n * ctx.n), loop)
    return builder.finalize()


if __name__ == "__main__":
    n, m = [int(x) for x in input().strip().split()]
    g = [0] * (n * n)
    for _ in range(m):
        a, b = [int(x) for x in input().strip().split()]
        g[a * n + b] += 1
        g[b * n + a] += 1
    program = mul(n)
    executor = wy.WyVkExecutor()
    executable = executor.compile(program)
    dev_a = executor.newResource()
    dev_b = executor.newResource()
    dev_c = executor.newResource()
    dev_n = executor.newResource()
    dev_a.set_data_array_uint32(g)
    dev_b.set_data_array_uint32(g)
    dev_c.set_data_array_uint32([0] * (n * n))
    dev_n.set_data_uint32(n)
    executable.bind("a", wy.IoType.input.value, dev_a)
    executable.bind("b", wy.IoType.input.value, dev_b)
    executable.bind("c", wy.IoType.output.value, dev_c)
    executable.bind("n", wy.IoType.input.value, dev_n)
    executable.run()
    executable.unbind("b", wy.IoType.input.value)
    executable.unbind("c", wy.IoType.output.value)
    executable.bind("b", wy.IoType.input.value, dev_c)
    executable.bind("c", wy.IoType.output.value, dev_b)
    executable.run()
    out = dev_b.get_data_array_uint32()
    acc = 0
    for i in range(n):
        acc += out[i * n + i]
    assert acc % 6 == 0
    print("Triangles:", acc // 6, file=sys.stderr)
