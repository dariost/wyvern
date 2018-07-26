#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <stdint.h>
#include <sys/time.h>

#ifndef BLOCK_SIZE
#define BLOCK_SIZE 128
#endif
#ifndef BLOCK_COUNT
#define BLOCK_COUNT 128
#endif
#define CENTER_X -0.75
#define CENTER_Y 0.0
#define ZOOM (float(height) / 2.5)

__global__ void mandelbrot(unsigned* dim, float* output, unsigned iterations) {
    unsigned width = dim[0];
    unsigned height = dim[1];
    unsigned tid = blockDim.x * blockIdx.x + threadIdx.x;
    for(; tid < width * height; tid += blockDim.x * gridDim.x) {
        float x = tid % width;
        float y = tid / width;
        x -= width / 2.0;
        y -= height / 2.0;
        x /= ZOOM;
        y /= ZOOM;
        x += CENTER_X;
        y += CENTER_Y;
        float a = 0.0, b = 0.0;
        for(unsigned i = 0; i < iterations; i++) {
            float tmp_a = a * a - b * b + x;
            b = 2.0 * a * b + y;
            a = tmp_a;
        }
        output[tid] = a * a + b * b;
    }
}

int main(int argc, char* argv[]) {
    assert(argc == 4);
    unsigned WIDTH, HEIGHT, ITERATIONS;
    WIDTH = atoi(argv[1]);
    HEIGHT = atoi(argv[2]);
    ITERATIONS = atoi(argv[3]);
    unsigned* host_dim;
    float* host_output;
    unsigned* device_dim;
    float* device_output;
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC_RAW, &start);
    host_dim = (unsigned*)malloc(2 * sizeof(unsigned));
    assert(host_dim);
    host_output = (float*)malloc(WIDTH * HEIGHT * sizeof(float));
    assert(host_output);
    cudaMalloc(&device_dim, 2 * sizeof(unsigned));
    cudaMalloc(&device_output, WIDTH * HEIGHT * sizeof(float));
    host_dim[0] = WIDTH;
    host_dim[1] = HEIGHT;
    cudaMemcpy(device_dim, host_dim, 2 * sizeof(unsigned), cudaMemcpyHostToDevice);
    mandelbrot<<<BLOCK_COUNT, BLOCK_SIZE>>>(device_dim, device_output, ITERATIONS);
    cudaMemcpy(host_output, device_output, WIDTH * HEIGHT * sizeof(float), cudaMemcpyDeviceToHost);
    cudaFree(device_output);
    cudaFree(device_dim);
    clock_gettime(CLOCK_MONOTONIC_RAW, &end);
    FILE* output = fopen("out.ppm", "w");
    fprintf(output, "P2\n%u %u\n255\n", host_dim[0], host_dim[1]);
    for(unsigned i = 0; i < WIDTH * HEIGHT; i++) {
        fprintf(output, "%d\n", (host_output[i] <= 2.0) ? (0) : (255));
    }
    fclose(output);
    free(host_dim);
    free(host_output);
    uint64_t delta_us = (end.tv_sec - start.tv_sec) * 1000000 + (end.tv_nsec - start.tv_nsec) / 1000;
    double delta = double(delta_us) / 1e6;
    printf("%.9lf\n", delta);
    return 0;
}
