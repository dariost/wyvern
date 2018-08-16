#![feature(use_extern_macros, specialization)]

extern crate pyo3;
extern crate serde_json;
extern crate wyvern;

use wyvern::vk::resource::VkResource;
use std::sync::Arc;
use wyvern::core::executor::{Executor, Executable, Resource, IO};
use pyo3::prelude::*;
use wyvern::core::program::Program;
use wyvern::vk::executor::VkExecutor;
use wyvern::vk::executable::VkExecutable;
use wyvern::core::program::{ConstantScalar, ConstantVector, TokenValue};

#[pyclass]
struct WyVkExecutor {
    token: PyToken,
    data: VkExecutor
}

#[pyclass]
struct WyVkExecutable {
    token: PyToken,
    data: VkExecutable
}

#[pyclass]
struct WyVkResource {
    token: PyToken,
    data: Arc<VkResource>
}

#[pymethods]
impl WyVkResource {
    fn set_data_uint32(&self, value: u32) -> PyResult<()> {
        self.data.set_data(TokenValue::Scalar(ConstantScalar::U32(value)));
        Ok(())
    }

    fn set_data_int32(&self, value: i32) -> PyResult<()> {
        self.data.set_data(TokenValue::Scalar(ConstantScalar::I32(value)));
        Ok(())
    }

    fn set_data_float32(&self, value: f32) -> PyResult<()> {
        self.data.set_data(TokenValue::Scalar(ConstantScalar::F32(value)));
        Ok(())
    }

    fn set_data_array_uint32(&self, value: Vec<u32>) -> PyResult<()> {
        self.data.set_data(TokenValue::Vector(ConstantVector::U32(value)));
        Ok(())
    }

    fn set_data_array_int32(&self, value: Vec<i32>) -> PyResult<()> {
        self.data.set_data(TokenValue::Vector(ConstantVector::I32(value)));
        Ok(())
    }

    fn set_data_array_float32(&self, value: Vec<f32>) -> PyResult<()> {
        self.data.set_data(TokenValue::Vector(ConstantVector::F32(value)));
        Ok(())
    }

    fn get_data_uint32(&self) -> PyResult<u32> {
        if let TokenValue::Scalar(ConstantScalar::U32(value)) = self.data.get_data() {
            Ok(value)
        } else {
            panic!("Wrong type requested!");
        }
    }

    fn get_data_int32(&self) -> PyResult<i32> {
        if let TokenValue::Scalar(ConstantScalar::I32(value)) = self.data.get_data() {
            Ok(value)
        } else {
            panic!("Wrong type requested!");
        }
    }

    fn get_data_float32(&self) -> PyResult<f32> {
        if let TokenValue::Scalar(ConstantScalar::F32(value)) = self.data.get_data() {
            Ok(value)
        } else {
            panic!("Wrong type requested!");
        }
    }

    fn get_data_array_uint32(&self) -> PyResult<Vec<u32>> {
        if let TokenValue::Vector(ConstantVector::U32(value)) = self.data.get_data() {
            Ok(value)
        } else {
            panic!("Wrong type requested!");
        }
    }

    fn get_data_array_int32(&self) -> PyResult<Vec<i32>> {
        if let TokenValue::Vector(ConstantVector::I32(value)) = self.data.get_data() {
            Ok(value)
        } else {
            panic!("Wrong type requested!");
        }
    }

    fn get_data_array_float32(&self) -> PyResult<Vec<f32>> {
        if let TokenValue::Vector(ConstantVector::F32(value)) = self.data.get_data() {
            Ok(value)
        } else {
            panic!("Wrong type requested!");
        }
    }
}

#[pymethods]
impl WyVkExecutable {
    fn bind(&mut self, name: String, kind: String, resource: PyObject) -> PyResult<()> {
        let py = self.token.py();
        let kind = if kind == "input" {
            IO::Input
        } else if kind == "output" {
            IO::Output
        } else {
            unreachable!();
        };
        let resource: &WyVkResource = PyObject::extract(&resource, py).expect("PyO3 error");
        self.data.bind(name, kind, resource.data.clone());
        Ok(())
    }

    fn unbind(&mut self, name: String, kind: String) -> PyResult<()> {
        let kind = if kind == "input" {
            IO::Input
        } else if kind == "output" {
            IO::Output
        } else {
            unreachable!();
        };
        self.data.unbind(name, kind);
        Ok(())
    }

    fn run(&mut self) -> PyResult<()> {
        self.data.run().unwrap();
        Ok(())
    }
}

#[allow(non_snake_case)]
#[pymethods]
impl WyVkExecutor {
    #[new]
     fn __new__(obj: &PyRawObject) -> PyResult<()> {
         obj.init(|token| WyVkExecutor {
             token,
             data: VkExecutor::new(Default::default()).unwrap()
         })
     }

     fn compile(&self, program: String) -> PyResult<Py<WyVkExecutable>> {
         let py = self.token.py();
         let program: Program = serde_json::from_str(&program).unwrap();
         Py::new(py, |token| WyVkExecutable {
             token,
             data: self.data.compile(program).unwrap()
         })
     }

     fn newResource(&self) -> PyResult<Py<WyVkResource>> {
         let py = self.token.py();
         Py::new(py, |token| WyVkResource {
             token,
             data: self.data.new_resource().unwrap()
         })
     }
}

#[pymodinit(libwyvern)]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<WyVkExecutor>().unwrap();
    m.add_class::<WyVkExecutable>().unwrap();
    m.add_class::<WyVkResource>().unwrap();
    Ok(())
}
