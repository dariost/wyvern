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

__global__ void matmul(const unsigned n, const unsigned* a, const unsigned* b, unsigned* c) {
    const unsigned tsize = blockDim.x * gridDim.x;
    for(unsigned tid = blockDim.x * blockIdx.x + threadIdx.x; tid < n * n; tid += tsize) {
        const unsigned i = tid % n;
        const unsigned j = tid / n;
        unsigned acc = 0;
        for(unsigned k = 0; k < n; k++) {
            acc += a[i * n + k] * b[k * n + j];
        }
        c[i * n + j] = acc;
    }
}

int main(int argc, char* argv[]) {
    assert(argc == 2);
    freopen(argv[1], "r", stdin);
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC_RAW, &start);
    unsigned* h_a,* h_b,* h_c,* d_a,* d_b,* d_c;
    unsigned n, m, trace = 0;
    scanf("%u%u", &n, &m);
    h_a = (unsigned*)calloc(n * n, sizeof(unsigned));
    h_b = (unsigned*)calloc(n * n, sizeof(unsigned));
    h_c = (unsigned*)calloc(n * n, sizeof(unsigned));
    for(unsigned i = 0; i < m; i++) {
        unsigned a, b;
        scanf("%u%u", &a, &b);
        h_a[a * n + b] = h_b[a * n + b] = 1;
        h_a[b * n + a] = h_b[b * n + a] = 1;
    }
    cudaMalloc(&d_a, n * n * sizeof(unsigned));
    cudaMalloc(&d_b, n * n * sizeof(unsigned));
    cudaMalloc(&d_c, n * n * sizeof(unsigned));
    cudaMemcpy(d_a, h_a, n * n * sizeof(unsigned), cudaMemcpyHostToDevice);
    cudaMemcpy(d_b, h_b, n * n * sizeof(unsigned), cudaMemcpyHostToDevice);
    cudaMemcpy(d_c, h_c, n * n * sizeof(unsigned), cudaMemcpyHostToDevice);
    matmul<<<BLOCK_COUNT, BLOCK_SIZE>>>(n, d_a, d_b, d_c);
    matmul<<<BLOCK_COUNT, BLOCK_SIZE>>>(n, d_a, d_c, d_b);
    cudaMemcpy(h_c, d_b, n * n * sizeof(unsigned), cudaMemcpyDeviceToHost);
    cudaFree(d_a);
    cudaFree(d_b);
    cudaFree(d_c);
    for(unsigned i = 0; i < n; i++) {
        trace += h_c[i * n + i];
    }
    free(h_a);
    free(h_b);
    free(h_c);
    assert(trace % 6 == 0);
    clock_gettime(CLOCK_MONOTONIC_RAW, &end);
    uint64_t delta_us = (end.tv_sec - start.tv_sec) * 1000000 + (end.tv_nsec - start.tv_nsec) / 1000;
    double delta = double(delta_us) / 1e6;
    printf("%.9lf\n", delta);
    fprintf(stderr, "Triangles: %u\n", trace / 6);
    return 0;
}
