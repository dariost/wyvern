extern crate pyo3;
extern crate serde_json;
extern crate wyvern;

use pyo3::prelude::*;
use std::sync::Arc;
use wyvern::core::executor::{Executable, Executor, Resource, IO};
use wyvern::core::program::Program;
use wyvern::core::program::{ConstantScalar, ConstantVector, TokenValue};
use wyvern::vk::executable::VkExecutable;
use wyvern::vk::executor::VkExecutor;
use wyvern::vk::resource::VkResource;

#[pyclass]
struct WyVkExecutor {
    data: VkExecutor,
}

#[pyclass]
struct WyVkExecutable {
    data: VkExecutable,
}

#[pyclass]
struct WyVkResource {
    data: Arc<VkResource>,
}

#[pymethods]
impl WyVkResource {
    fn set_data_uint32(&self, value: u32) -> PyResult<()> {
        self.data
            .set_data(TokenValue::Scalar(ConstantScalar::U32(value)));
        Ok(())
    }

    fn set_data_int32(&self, value: i32) -> PyResult<()> {
        self.data
            .set_data(TokenValue::Scalar(ConstantScalar::I32(value)));
        Ok(())
    }

    fn set_data_float32(&self, value: f32) -> PyResult<()> {
        self.data
            .set_data(TokenValue::Scalar(ConstantScalar::F32(value)));
        Ok(())
    }

    fn set_data_array_uint32(&self, value: Vec<u32>) -> PyResult<()> {
        self.data
            .set_data(TokenValue::Vector(ConstantVector::U32(value)));
        Ok(())
    }

    fn set_data_array_int32(&self, value: Vec<i32>) -> PyResult<()> {
        self.data
            .set_data(TokenValue::Vector(ConstantVector::I32(value)));
        Ok(())
    }

    fn set_data_array_float32(&self, value: Vec<f32>) -> PyResult<()> {
        self.data
            .set_data(TokenValue::Vector(ConstantVector::F32(value)));
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
    fn bind(&mut self, name: String, kind: String, resource: &WyVkResource) -> PyResult<()> {
        let kind = if kind == "input" {
            IO::Input
        } else if kind == "output" {
            IO::Output
        } else {
            unreachable!();
        };
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
    fn __new__() -> PyResult<WyVkExecutor> {
        Ok(WyVkExecutor {
            data: VkExecutor::new(Default::default()).unwrap(),
        })
    }

    fn compile(&self, program: String) -> PyResult<WyVkExecutable> {
        let program: Program = serde_json::from_str(&program).unwrap();
        Ok(WyVkExecutable {
            data: self.data.compile(program).unwrap(),
        })
    }

    fn newResource(&self) -> PyResult<WyVkResource> {
        Ok(WyVkResource {
            data: self.data.new_resource().unwrap(),
        })
    }
}

#[pymodule]
fn init_mod(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<WyVkExecutor>()?;
    m.add_class::<WyVkExecutable>()?;
    m.add_class::<WyVkResource>()?;
    Ok(())
}
