#include <stdlib.h>
#include <stdio.h>
#include <assert.h>
#include <time.h>
#include <CL/cl.h>

const char* source = ""
"#define CENTER_X -0.75\n"
"#define CENTER_Y 0.0\n"
"#define ZOOM (height / 2.5)\n"
"\n"
"__kernel void mandelbrot(__global const uint* dim, __global float* output) {\n"
"    size_t tid = get_global_id(0);\n"
"    size_t tsize = get_global_size(0);\n"
"    unsigned width = dim[0];\n"
"    unsigned height = dim[1];\n"
"    unsigned iterations = dim[2];\n"
"    for(; tid < width * height; tid += tsize) {\n"
"        float x = tid % width;\n"
"        float y = tid / width;\n"
"        x -= width / 2.0;\n"
"        y -= height / 2.0;\n"
"        x /= ZOOM;\n"
"        y /= ZOOM;\n"
"        x += CENTER_X;\n"
"        y += CENTER_Y;\n"
"        float a = 0.0, b = 0.0;\n"
"        for(unsigned i = 0; i < iterations; i++) {\n"
"            float tmp_a = a * a - b * b + x;\n"
"            b = 2.0 * a * b + y;\n"
"            a = tmp_a;\n"
"        }\n"
"        output[tid] = a * a + b * b;\n"
"    }\n"
"}\n"
"";

int main(int argc, char* argv[]) {
    assert(argc == 4);
    struct timespec start, end;
    unsigned width = atoi(argv[1]);
    unsigned height = atoi(argv[2]);
    unsigned iterations = atoi(argv[3]);
    unsigned* host_params = calloc(2, sizeof(unsigned));
    host_params[0] = width;
    host_params[1] = height;
    host_params[2] = iterations;
    float* host_output = calloc(width * height, sizeof(float));
    clock_gettime(CLOCK_MONOTONIC_RAW, &start);
    cl_mem device_params, device_output;
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
    kernel = clCreateKernel(program, "mandelbrot", NULL);
    device_params = clCreateBuffer(context, CL_MEM_READ_ONLY, sizeof(unsigned) * 3, NULL, NULL);
    device_output = clCreateBuffer(context, CL_MEM_WRITE_ONLY, sizeof(float) * width * height, NULL, NULL);
    clEnqueueWriteBuffer(commands, device_params, CL_TRUE, 0, sizeof(unsigned) * 3, host_params, 0, NULL, NULL);
    clSetKernelArg(kernel, 0, sizeof(cl_mem), &device_params);
    clSetKernelArg(kernel, 1, sizeof(cl_mem), &device_output);
    size_t local_size;
    clGetKernelWorkGroupInfo(kernel, device_id, CL_KERNEL_WORK_GROUP_SIZE, sizeof(local_size), &local_size, NULL);
    const size_t global_size = 4096;
    clEnqueueNDRangeKernel(commands, kernel, 1, NULL, &global_size, &local_size, 0, NULL, NULL);
    clFinish(commands);
    clEnqueueReadBuffer(commands, device_output, CL_TRUE, 0, sizeof(float) * width * height, host_output, 0, NULL, NULL);
    clock_gettime(CLOCK_MONOTONIC_RAW, &end);
    clReleaseMemObject(device_params);
    clReleaseMemObject(device_output);
    clReleaseProgram(program);
    clReleaseKernel(kernel);
    clReleaseCommandQueue(commands);
    clReleaseContext(context);
    FILE* output = fopen("out.ppm", "w");
    fprintf(output, "P2\n%u %u\n255\n", host_params[0], host_params[1]);
    for(unsigned i = 0; i < width * height; i++) {
        fprintf(output, "%d\n", (host_output[i] <= 2.0) ? (0) : (255));
    }
    fclose(output);
    free(host_params);
    free(host_output);
    uint64_t delta_us = (end.tv_sec - start.tv_sec) * 1000000 + (end.tv_nsec - start.tv_nsec) / 1000;
    double delta = ((double)delta_us) / 1e6;
    printf("%.9lf\n", delta);
    return 0;
}
