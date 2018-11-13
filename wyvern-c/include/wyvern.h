#ifndef _WYVERN_H_
#define _WYVERN_H_

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define WYVERN_INPUT 0
#define WYVERN_OUTPUT 1

typedef void wyvern_vk_executor_t;
typedef void wyvern_vk_executable_t;
typedef void wyvern_vk_resource_t;

typedef struct {
    uint32_t size;
    uint32_t data[];
} wyvern_data_array_uint32_t;
typedef struct {
    uint32_t size;
    int32_t data[];
} wyvern_data_array_int32_t;
typedef struct {
    uint32_t size;
    float data[];
} wyvern_data_array_float32_t;

wyvern_vk_executor_t* wyvern_vk_executor_new();
void wyvern_vk_executor_destroy(wyvern_vk_executor_t* obj);

wyvern_vk_executable_t* wyvern_vk_executable_new(wyvern_vk_executor_t* obj, const char* source);
void wyvern_vk_executable_destroy(wyvern_vk_executable_t* obj);

wyvern_vk_resource_t* wyvern_vk_resource_new(wyvern_vk_executor_t* obj);
void wyvern_vk_resource_destroy(wyvern_vk_resource_t* obj);

void wyvern_vk_executable_bind(wyvern_vk_executable_t* obj, const char* name, uint32_t io, wyvern_vk_resource_t* resource);
void wyvern_vk_executable_unbind(wyvern_vk_executable_t* obj, const char* name, uint32_t io);
void wyvern_vk_executable_run(wyvern_vk_executable_t* obj);

void wyvern_vk_resource_set_data_uint32(wyvern_vk_resource_t* obj, uint32_t data);
void wyvern_vk_resource_set_data_int32(wyvern_vk_resource_t* obj, int32_t data);
void wyvern_vk_resource_set_data_float32(wyvern_vk_resource_t* obj, float data);

uint32_t wyvern_vk_resource_get_data_uint32(wyvern_vk_resource_t* obj);
int32_t wyvern_vk_resource_get_data_int32(wyvern_vk_resource_t* obj);
float wyvern_vk_resource_get_data_float32(wyvern_vk_resource_t* obj);

void wyvern_vk_resource_set_data_array_uint32(wyvern_vk_resource_t* obj, const uint32_t* data, size_t n_elements);
void wyvern_vk_resource_set_data_array_int32(wyvern_vk_resource_t* obj, const int32_t* data, size_t n_elements);
void wyvern_vk_resource_set_data_array_float32(wyvern_vk_resource_t* obj, const float* data, size_t n_elements);

wyvern_data_array_uint32_t* wyvern_vk_resource_get_data_array_uint32(wyvern_vk_resource_t* obj);
wyvern_data_array_int32_t* wyvern_vk_resource_get_data_array_int32(wyvern_vk_resource_t* obj);
wyvern_data_array_float32_t* wyvern_vk_resource_get_data_array_float32(wyvern_vk_resource_t* obj);

void wyvern_vk_resource_data_array_uint32_free(wyvern_data_array_uint32_t* obj);
void wyvern_vk_resource_data_array_int32_free(wyvern_data_array_int32_t* obj);
void wyvern_vk_resource_data_array_float32_free(wyvern_data_array_float32_t* obj);

#ifdef __cplusplus
}
#endif

#endif
