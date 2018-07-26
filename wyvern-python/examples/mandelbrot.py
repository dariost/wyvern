#!/usr/bin/env python3

import sys
import os

sys.path.append(os.path.abspath("./"))
sys.path.append(os.path.abspath("../"))

import wyvern as wy
from time import time
import png


def mandelbrot(g_ctx: wy.builder.Context, id: str, a0: str, b0: str,
               iterations: int):
    ctx = g_ctx.getProgramBuilder().newContext()
    ctx.a = 0.0
    ctx.b = 0.0
    ctx.i = 0

    def loop():
        ctx.tmp_a = ctx.a * ctx.a - ctx.b * ctx.b + g_ctx[a0]
        ctx.b = ctx.a * ctx.b * 2.0 + g_ctx[b0]
        ctx.a = ctx.tmp_a
        ctx.i = ctx.i + 1

    ctx.While(lambda: ctx.i < iterations, loop)
    g_ctx[id] = ctx.a * ctx.a + ctx.b * ctx.b


def pixel2coordinates(g_ctx: wy.builder.Context, x: str, y: str, x0: str,
                      y0: str, out_x: str, out_y: str, width: wy.types.Constant,
                      height: wy.types.Constant, zoom: float):
    g_ctx[x] = g_ctx[x] - width / 2.0
    g_ctx[y] = g_ctx[y] - height / 2.0
    g_ctx[x] = g_ctx[x] / zoom
    g_ctx[y] = g_ctx[y] / zoom
    g_ctx[out_x] = g_ctx[x] + g_ctx[x0]
    g_ctx[out_y] = g_ctx[y] + g_ctx[y0]


def program(size: int, center_x: float, center_y: float, zoom: float,
            iterations: int) -> str:
    builder = wy.ProgramBuilder()
    ctx = builder.newContext()
    ctx.declArray("input", wy.DataType.uint32, wy.IoType.input,
                  wy.types.Constant.uint32(2, ctx), 2)
    ctx.width = ctx.input[0]
    ctx.height = ctx.input[1]
    ctx.declArray("output", wy.DataType.float32, wy.IoType.output,
                  ctx.width * ctx.height, size)
    ctx.id = ctx.workerId()
    ctx.size = ctx.numWorkers()
    ctx.center_x = wy.types.Constant.float32(center_x, ctx)
    ctx.center_y = wy.types.Constant.float32(center_y, ctx)

    def loop():
        ctx.x = wy.types.Constant.float32(ctx.id % ctx.width)
        ctx.y = wy.types.Constant.float32(ctx.id // ctx.width)
        pixel2coordinates(ctx, "x", "y", "center_x", "center_y", "x", "y",
                          wy.types.Constant.float32(ctx.input[0]),
                          wy.types.Constant.float32(ctx.input[1]), zoom)
        mandelbrot(ctx, "out", "x", "y", iterations)
        ctx.output[ctx.id] = ctx.out
        ctx.id = ctx.id + ctx.size

    ctx.While(lambda: ctx.id < ctx.width * ctx.height, loop)
    return builder.finalize()


if __name__ == "__main__":
    assert len(sys.argv) == 4
    WIDTH = int(sys.argv[1])
    HEIGHT = int(sys.argv[2])
    ITERATIONS = int(sys.argv[3])
    result = program(WIDTH * HEIGHT, -0.75, 0.0, HEIGHT / 2.5, ITERATIONS)
    executor = wy.WyVkExecutor()
    executable = executor.compile(result)
    start = time()
    input = executor.newResource()
    output = executor.newResource()
    input.set_data_array_uint32([WIDTH, HEIGHT])
    output.set_data_array_float32([0.0] * WIDTH * HEIGHT)
    executable.bind("input", wy.IoType.input.value, input)
    executable.bind("output", wy.IoType.output.value, output)
    executable.run()
    result = output.get_data_array_float32()
    print("%.9f" % (time() - start,))

    def mapper(x):
        if x <= 2.0:
            return 0
        else:
            return 255
    result = [mapper(x) for x in result]
    out = open("out.png", "wb")
    w = png.Writer(WIDTH, HEIGHT, greyscale=True)
    w.write_array(out, result)
    out.close()
