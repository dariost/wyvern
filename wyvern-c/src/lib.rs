#![allow(non_camel_case_types)]

extern crate libc;
extern crate serde_json;
extern crate wyvern;

use libc::{free, malloc};
use std::ffi::CStr;
use std::ffi::c_void;
use std::mem;
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

#[repr(C)]
pub struct wyvern_data_array_uint32_t {
    size: u32,
    data: *const u32,
}

#[repr(C)]
pub struct wyvern_data_array_int32_t {
    size: u32,
    data: *const i32,
}

#[repr(C)]
pub struct wyvern_data_array_float_t {
    size: u32,
    data: *const f32,
}

pub type wyvern_vk_executor_t = c_void;
pub type wyvern_vk_executable_t = c_void;
pub type wyvern_vk_resource_t = c_void;

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_executor_new() -> *mut wyvern_vk_executor_t {
    let executor = Box::new(VkExecutor::new(Default::default()).unwrap());
    Box::into_raw(executor) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_executor_destroy(obj: *mut wyvern_vk_executor_t) {
    Box::from_raw(obj as *mut VkExecutor);
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_executable_new(
    obj: *mut wyvern_vk_executor_t,
    source: *const c_char,
) -> *mut wyvern_vk_executable_t {
    let obj = &mut *(obj as *mut VkExecutor);
    let source = CStr::from_ptr(source).to_str().unwrap();
    let program: Program = serde_json::from_str(source).unwrap();
    let executable = Box::new(obj.compile(program).unwrap());
    Box::into_raw(executable) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_executable_destroy(obj: *mut wyvern_vk_executable_t) {
    Box::from_raw(obj as *mut VkExecutable);
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_new(obj: *mut wyvern_vk_executor_t) -> *mut wyvern_vk_resource_t {
    let obj = &mut *(obj as *mut VkExecutor);
    let resource = Box::new(obj.new_resource().unwrap());
    Box::into_raw(resource) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_destroy(obj: *mut wyvern_vk_resource_t) {
    Box::from_raw(obj as *mut Arc<VkResource>);
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_executable_bind(
    obj: *mut wyvern_vk_executable_t,
    name: *const c_char,
    kind: u32,
    resource: *mut wyvern_vk_resource_t,
) {
    let obj = &mut *(obj as *mut VkExecutable);
    let name = CStr::from_ptr(name).to_str().unwrap();
    let kind = match kind {
        WYVERN_INPUT => IO::Input,
        WYVERN_OUTPUT => IO::Output,
        _ => panic!(),
    };
    let resource = &mut *(resource as *mut Arc<VkResource>);
    obj.bind(name, kind, resource.clone());
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_executable_unbind(obj: *mut wyvern_vk_executable_t, name: *const c_char, kind: u32) {
    let obj = &mut *(obj as *mut VkExecutable);
    let name = CStr::from_ptr(name).to_str().unwrap();
    let kind = match kind {
        WYVERN_INPUT => IO::Input,
        WYVERN_OUTPUT => IO::Output,
        _ => panic!("Invalid constant!"),
    };
    obj.unbind(name, kind);
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_executable_run(obj: *mut wyvern_vk_executable_t) {
    let obj = &mut *(obj as *mut VkExecutable);
    obj.run().unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_set_data_uint32(obj: *mut wyvern_vk_resource_t, data: u32) {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    obj.set_data(TokenValue::Scalar(ConstantScalar::U32(data)));
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_set_data_int32(obj: *mut wyvern_vk_resource_t, data: i32) {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    obj.set_data(TokenValue::Scalar(ConstantScalar::I32(data)));
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_set_data_float32(obj: *mut wyvern_vk_resource_t, data: f32) {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    obj.set_data(TokenValue::Scalar(ConstantScalar::F32(data)));
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_set_data_array_uint32(
    obj: *mut wyvern_vk_resource_t,
    data: *const u32,
    n_elem: u32,
) {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    let data = slice::from_raw_parts(data, n_elem as usize);
    obj.set_data(TokenValue::Vector(ConstantVector::U32(data.to_vec())));
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_set_data_array_int32(
    obj: *mut wyvern_vk_resource_t,
    data: *const i32,
    n_elem: u32,
) {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    let data = slice::from_raw_parts(data, n_elem as usize);
    obj.set_data(TokenValue::Vector(ConstantVector::I32(data.to_vec())));
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_set_data_array_float32(
    obj: *mut wyvern_vk_resource_t,
    data: *const f32,
    n_elem: u32,
) {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    let data = slice::from_raw_parts(data, n_elem as usize);
    obj.set_data(TokenValue::Vector(ConstantVector::F32(data.to_vec())));
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_get_data_uint32(obj: *mut wyvern_vk_resource_t) -> u32 {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    if let TokenValue::Scalar(ConstantScalar::U32(value)) = obj.get_data() {
        value
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_get_data_int32(obj: *mut wyvern_vk_resource_t) -> i32 {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    if let TokenValue::Scalar(ConstantScalar::I32(value)) = obj.get_data() {
        value
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_get_data_float32(obj: *mut wyvern_vk_resource_t) -> f32 {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    if let TokenValue::Scalar(ConstantScalar::F32(value)) = obj.get_data() {
        value
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_get_data_array_uint32(obj: *mut wyvern_vk_resource_t) -> *mut wyvern_data_array_uint32_t {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    if let TokenValue::Vector(ConstantVector::U32(value)) = obj.get_data() {
        let data = malloc(value.len() * 4) as *mut u32;
        for i in 0..value.len() {
            write(data.offset(i as isize), value[i]);
        }
        let arr = wyvern_data_array_uint32_t {
            size: value.len() as u32,
            data: data,
        };
        let arr_p = malloc(mem::size_of::<wyvern_data_array_uint32_t>()) as *mut wyvern_data_array_uint32_t;
        write(arr_p, arr);
        arr_p
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_get_data_array_int32(obj: *mut wyvern_vk_resource_t) -> *mut wyvern_data_array_int32_t {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    if let TokenValue::Vector(ConstantVector::I32(value)) = obj.get_data() {
        let data = malloc(value.len() * 4) as *mut i32;
        for i in 0..value.len() {
            write(data.offset(i as isize), value[i]);
        }
        let arr = wyvern_data_array_int32_t {
            size: value.len() as u32,
            data: data,
        };
        let arr_p = malloc(mem::size_of::<wyvern_data_array_int32_t>()) as *mut wyvern_data_array_int32_t;
        write(arr_p, arr);
        arr_p
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_get_data_array_float32(obj: *mut wyvern_vk_resource_t) -> *mut wyvern_data_array_float_t {
    let obj = &mut *(obj as *mut Arc<VkResource>);
    if let TokenValue::Vector(ConstantVector::F32(value)) = obj.get_data() {
        let data = malloc(value.len() * 4) as *mut f32;
        for i in 0..value.len() {
            write(data.offset(i as isize), value[i]);
        }
        let arr = wyvern_data_array_float_t {
            size: value.len() as u32,
            data: data,
        };
        let arr_p = malloc(mem::size_of::<wyvern_data_array_float_t>()) as *mut wyvern_data_array_float_t;
        write(arr_p, arr);
        arr_p
    } else {
        panic!("Wrong type requested!");
    }
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_data_array_uint32_free(data: *mut wyvern_data_array_uint32_t) {
    let array = (&mut *data).data;
    free(array as *mut libc::c_void);
    free(data as *mut libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_data_array_int32_free(data: *mut wyvern_data_array_int32_t) {
    let array = (&mut *data).data;
    free(array as *mut libc::c_void);
    free(data as *mut libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn wyvern_vk_resource_data_array_float32_free(data: *mut wyvern_data_array_float_t) {
    let array = (&mut *data).data;
    free(array as *mut libc::c_void);
    free(data as *mut libc::c_void);
}
