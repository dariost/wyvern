#![feature(proc_macro, specialization)]

extern crate pyo3;
extern crate serde_json;
extern crate wyvern;
use pyo3::prelude::*;
use wyvern::core::program::Program;

use pyo3::py::modinit as pymodinit;

#[pymodinit(libwyvern)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "printer")]
    fn printer(x: String) -> PyResult<String> {
        let x: Program = serde_json::from_str(&x).unwrap();
        Ok(format!("{:?}", x))
    }
    Ok(())
}
