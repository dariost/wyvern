#!/bin/bash -xe

cd "${0%/*}"
ln -sf ../wyvern-python/wyvern/ wyvern

pushd ..

pushd wyvern-python
cargo build --release
pushd wyvern
ln -sf ../target/release/libwyvern.so libwyvern.so
popd
popd

cargo build --release --example mandelbrot

popd

pushd mandelbrot
ln -sf ../../wyvern-python/examples/mandelbrot.py mpython
ln -sf ../../target/release/examples/mandelbrot mrust
nvcc -O3 ../../utility/mandelbrot.cu -o mcuda
gcc -O3 ../../utility/mandelbrot.c -lOpenCL -Wno-deprecated-declarations -o mopencl
popd

pushd triangles
mkdir -p input
ln -sf ../../utility/triangles.py tnumpy
ln -sf ../../wyvern-python/examples/triangles.py twyvern
nvcc -O3 ../../utility/triangles.cu -o tcuda
gcc -O3 ../../utility/triangles.c -lOpenCL -Wno-deprecated-declarations -o topencl
g++ ../../utility/triangles.cpp -O3 -fopenmp -o tmtcpu
g++ ../../utility/triangles.cpp -O3 -o tcpu

../../utility/gen_dense_grid.py 4 4 > input/16
../../utility/gen_dense_grid.py 4 32 > input/128
../../utility/gen_dense_grid.py 3 128 > input/384
../../utility/gen_dense_grid.py 6 128 > input/768
../../utility/gen_dense_grid.py 9 128 > input/1152
../../utility/gen_dense_grid.py 12 128 > input/1536
popd
