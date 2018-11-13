extern crate libc;
extern crate serde_json;
extern crate wyvern;

use libc::{free, malloc};
use std::ffi::c_void;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::write;
use std::slice;
use std::sync::Arc;
use wyvern::core::executor::{Executable, Executor, Resource, IO};
use wyvern::core::program::Program;
use wyvern::core::program::{ConstantScalar, ConstantVector, TokenValue};
use wyvern::vk::executable::VkExecutable;
use wyvern::vk::executor::VkExecutor;
use wyvern::vk::resource::VkResource;

pub const WYVERN_INPUT: u32 = 0;
pub const WYVERN_OUTPUT: u32 = 1;

#[no_mangle]
pub unsafe fn wyvern_vk_executor_new() -> *mut VkExecutor {
    let executor = Box::new(VkExecutor::new(Default::default()).unwrap());
    Box::into_raw(executor)
}

#[no_mangle]
pub unsafe fn wyvern_vk_executor_destroy(obj: *mut VkExecutor) {
    Box::from_raw(obj);
}

#[no_mangle]
pub unsafe fn wyvern_vk_executable_new(
    obj: *mut VkExecutor,
    source: *const c_char,
) -> *mut VkExecutable {
    let obj = &mut *obj;
    let source = CStr::from_ptr(source).to_str().unwrap();
    let program: Program = serde_json::from_str(source).unwrap();
    let executable = Box::new(obj.compile(program).unwrap());
    Box::into_raw(executable)
}

#[no_mangle]
pub unsafe fn wyvern_vk_executable_destroy(obj: *mut VkExecutor) {
    Box::from_raw(obj);
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_new(obj: *mut VkExecutor) -> *mut Arc<VkResource> {
    let obj = &mut *obj;
    let resource = Box::new(obj.new_resource().unwrap());
    Box::into_raw(resource)
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_destroy(obj: *mut Arc<VkResource>) {
    Box::from_raw(obj);
}

#[no_mangle]
pub unsafe fn wyvern_vk_executable_bind(
    obj: *mut VkExecutable,
    name: *const c_char,
    kind: u32,
    resource: *mut Arc<VkResource>,
) {
    let obj = &mut *obj;
    let name = CStr::from_ptr(name).to_str().unwrap();
    let kind = match kind {
        WYVERN_INPUT => IO::Input,
        WYVERN_OUTPUT => IO::Output,
        _ => panic!(),
    };
    let resource = &mut *resource;
    obj.bind(name, kind, resource.clone());
}

#[no_mangle]
pub unsafe fn wyvern_vk_executable_unbind(obj: *mut VkExecutable, name: *const c_char, kind: u32) {
    let obj = &mut *obj;
    let name = CStr::from_ptr(name).to_str().unwrap();
    let kind = match kind {
        WYVERN_INPUT => IO::Input,
        WYVERN_OUTPUT => IO::Output,
        _ => panic!("Invalid constant!"),
    };
    obj.unbind(name, kind);
}

#[no_mangle]
pub unsafe fn wyvern_vk_executable_run(obj: *mut VkExecutable) {
    let obj = &mut *obj;
    obj.run().unwrap();
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_set_data_uint32(obj: *mut Arc<VkResource>, data: u32) {
    let obj = &mut *obj;
    obj.set_data(TokenValue::Scalar(ConstantScalar::U32(data)));
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_set_data_int32(obj: *mut Arc<VkResource>, data: i32) {
    let obj = &mut *obj;
    obj.set_data(TokenValue::Scalar(ConstantScalar::I32(data)));
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_set_data_float32(obj: *mut Arc<VkResource>, data: f32) {
    let obj = &mut *obj;
    obj.set_data(TokenValue::Scalar(ConstantScalar::F32(data)));
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_set_data_array_uint32(
    obj: *mut Arc<VkResource>,
    data: *const u32,
    n_elem: usize,
) {
    let obj = &mut *obj;
    let data = slice::from_raw_parts(data, n_elem);
    obj.set_data(TokenValue::Vector(ConstantVector::U32(data.to_vec())));
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_set_data_array_int32(
    obj: *mut Arc<VkResource>,
    data: *const i32,
    n_elem: usize,
) {
    let obj = &mut *obj;
    let data = slice::from_raw_parts(data, n_elem);
    obj.set_data(TokenValue::Vector(ConstantVector::I32(data.to_vec())));
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_set_data_array_float32(
    obj: *mut Arc<VkResource>,
    data: *const f32,
    n_elem: usize,
) {
    let obj = &mut *obj;
    let data = slice::from_raw_parts(data, n_elem);
    obj.set_data(TokenValue::Vector(ConstantVector::F32(data.to_vec())));
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_get_data_uint32(obj: *mut Arc<VkResource>) -> u32 {
    let obj = &mut *obj;
    if let TokenValue::Scalar(ConstantScalar::U32(value)) = obj.get_data() {
        value
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_get_data_int32(obj: *mut Arc<VkResource>) -> i32 {
    let obj = &mut *obj;
    if let TokenValue::Scalar(ConstantScalar::I32(value)) = obj.get_data() {
        value
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_get_data_float32(obj: *mut Arc<VkResource>) -> f32 {
    let obj = &mut *obj;
    if let TokenValue::Scalar(ConstantScalar::F32(value)) = obj.get_data() {
        value
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_get_data_array_uint32(obj: *mut Arc<VkResource>) -> *mut c_void {
    let obj = &mut *obj;
    if let TokenValue::Vector(ConstantVector::U32(value)) = obj.get_data() {
        let data = malloc((value.len() + 1) * 4) as *mut u32;
        write(data, value.len() as u32);
        for i in 0..value.len() {
            write(data.offset(i as isize + 1), value[i]);
        }
        data as *mut c_void
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_get_data_array_int32(obj: *mut Arc<VkResource>) -> *mut c_void {
    let obj = &mut *obj;
    if let TokenValue::Vector(ConstantVector::I32(value)) = obj.get_data() {
        let data = malloc((value.len() + 1) * 4) as *mut i32;
        write(data, value.len() as i32);
        for i in 0..value.len() {
            write(data.offset(i as isize + 1), value[i]);
        }
        data as *mut c_void
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_get_data_array_float32(obj: *mut Arc<VkResource>) -> *mut c_void {
    let obj = &mut *obj;
    if let TokenValue::Vector(ConstantVector::F32(value)) = obj.get_data() {
        let data = malloc((value.len() + 1) * 4) as *mut f32;
        write(data, f32::from_bits(value.len() as u32));
        for i in 0..value.len() {
            write(data.offset(i as isize + 1), value[i]);
        }
        data as *mut c_void
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_data_array_uint32_free(data: *mut c_void) {
    free(data as *mut libc::c_void);
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_data_array_int32_free(data: *mut c_void) {
    free(data as *mut libc::c_void);
}

#[no_mangle]
pub unsafe fn wyvern_vk_resource_data_array_float32_free(data: *mut c_void) {
    free(data as *mut libc::c_void);
}
