//! An implementation of the typemake interpreter using python.

use crate::error::{TypemakeError, TypemakeResult};
use crate::interpreter::Interpreter;
use pyo3::{PyErr, Python};
use std::fmt::Formatter;
use thiserror::Error;

/// A wrapper around the python interpreter provided by `pyo3`.
#[derive(Default)]
pub struct PythonInterpreter;

impl Interpreter for PythonInterpreter {
    fn run(&mut self, script: &str) -> TypemakeResult<()> {
        Python::with_gil(|py| {
            py.run(script, None, None)?;
            py.run(
                "import sys\nsys.stdout.flush()\nsys.stderr.flush()",
                None,
                None,
            )
        })
        .map_err(PythonInterpreterError::from)
        .map_err(TypemakeError::from)
    }

    fn version(&self) -> TypemakeResult<String> {
        Ok(Python::with_gil(|py| {
            format!("Python {}", py.version()).replace('\n', " ")
        }))
    }
}

/// A wrapper around the error type of the python interpreter provided by `pyo3`.
#[derive(Error, Debug)]
pub struct PythonInterpreterError(#[from] PyErr);

impl std::fmt::Display for PythonInterpreterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
