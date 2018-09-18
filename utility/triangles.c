#include <stdlib.h>
#include <stdio.h>
#include <assert.h>
#include <time.h>
#include <CL/cl.h>

const char* source = ""
"__kernel void matmul(const uint n, __global const uint* a,\n"
"                     __global const uint* b, __global uint* c) {\n"
"    size_t tid = get_global_id(0);\n"
"    const size_t tsize = get_global_size(0);\n"
"    for(; tid < n * n; tid += tsize) {\n"
"       const uint i = tid % n;\n"
"       const uint j = tid / n;\n"
"       uint acc = 0;\n"
"       for(uint k = 0; k < n; k++) {\n"
"          acc += a[i * n + k] * b[k * n + j];\n"
"       }\n"
"       c[i * n + j] = acc;\n"
"    }\n"
"}\n"
"";

int main(int argc, char* argv[]) {
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC_RAW, &start);
    unsigned* h_a,* h_b,* h_c;
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
    clock_gettime(CLOCK_MONOTONIC_RAW, &start);
    cl_mem d_a, d_b, d_c;
    cl_platform_id platform_id;
    cl_device_id device_id;
    cl_context context;
    cl_command_queue commands;
    cl_program program;
    cl_kernel kernel;
    clGetPlatformIDs(1, &platform_id, NULL);
    clGetDeviceIDs(platform_id, CL_DEVICE_TYPE_GPU, 1, &device_id, NULL);
    context = clCreateContext(0, 1, &device_id, NULL, NULL, NULL);
    commands = clCreateCommandQueue(context, device_id, 0, NULL);
    program = clCreateProgramWithSource(context, 1, &source, NULL, NULL);
    clBuildProgram(program, 0, NULL, NULL, NULL, NULL);
    kernel = clCreateKernel(program, "matmul", NULL);
    d_a = clCreateBuffer(context, CL_MEM_READ_ONLY, sizeof(unsigned) * n * n, NULL, NULL);
    d_b = clCreateBuffer(context, CL_MEM_READ_WRITE, sizeof(unsigned) * n * n, NULL, NULL);
    d_c = clCreateBuffer(context, CL_MEM_READ_WRITE, sizeof(unsigned) * n * n, NULL, NULL);
    clEnqueueWriteBuffer(commands, d_a, CL_FALSE, 0, sizeof(unsigned) * n * n, h_a, 0, NULL, NULL);
    clEnqueueWriteBuffer(commands, d_b, CL_FALSE, 0, sizeof(unsigned) * n * n, h_b, 0, NULL, NULL);
    clEnqueueWriteBuffer(commands, d_c, CL_FALSE, 0, sizeof(unsigned) * n * n, h_c, 0, NULL, NULL);
    clSetKernelArg(kernel, 0, sizeof(unsigned), &n);
    clSetKernelArg(kernel, 1, sizeof(cl_mem), &d_a);
    clSetKernelArg(kernel, 2, sizeof(cl_mem), &d_b);
    clSetKernelArg(kernel, 3, sizeof(cl_mem), &d_c);
    size_t local_size;
    clGetKernelWorkGroupInfo(kernel, device_id, CL_KERNEL_WORK_GROUP_SIZE, sizeof(local_size), &local_size, NULL);
    const size_t global_size = 1 << 13;
    clEnqueueNDRangeKernel(commands, kernel, 1, NULL, &global_size, &local_size, 0, NULL, NULL);
    clFlush(commands);
    clSetKernelArg(kernel, 2, sizeof(cl_mem), &d_c);
    clSetKernelArg(kernel, 3, sizeof(cl_mem), &d_b);
    clEnqueueNDRangeKernel(commands, kernel, 1, NULL, &global_size, &local_size, 0, NULL, NULL);
    clEnqueueReadBuffer(commands, d_b, CL_TRUE, 0, sizeof(unsigned) * n * n, h_c, 0, NULL, NULL);
    clReleaseMemObject(d_a);
    clReleaseMemObject(d_b);
    clReleaseMemObject(d_c);
    clReleaseProgram(program);
    clReleaseKernel(kernel);
    clReleaseCommandQueue(commands);
    clReleaseContext(context);
    for(unsigned i = 0; i < n; i++) {
        trace += h_c[i * n + i];
    }
    free(h_a);
    free(h_b);
    free(h_c);
    assert(trace % 6 == 0);
    clock_gettime(CLOCK_MONOTONIC_RAW, &end);
    uint64_t delta_us = (end.tv_sec - start.tv_sec) * 1000000 + (end.tv_nsec - start.tv_nsec) / 1000;
    double delta = ((double)delta_us) / 1e6;
    printf("%.9lf\n", delta);
    fprintf(stderr, "Triangles: %u\n", trace / 6);
    return 0;
}
