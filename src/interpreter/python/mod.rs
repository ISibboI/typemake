//! An implementation of the typemake interpreter using python.

use crate::error::{TypemakeError, TypemakeResult};
use crate::interpreter::Interpreter;
use pyo3::prelude::*;
use std::fmt::Formatter;
use thiserror::Error;
use std::sync::atomic::AtomicBool;
use log::{info, error};
use std::sync::Mutex;
use lazy_static::lazy_static;

static PYTHON_INTERPRETER_CREATED: AtomicBool = AtomicBool::new(false);

lazy_static!{
    static ref PYTHON_STDOUT: Mutex<String> = Mutex::new(String::new());
    static ref PYTHON_STDERR: Mutex<String> = Mutex::new(String::new());
}

//#[pymodule]
//#[pyo3(name = "typemake_internal")]
fn redirect_stdout_stderr(py: Python) -> PyResult<()> {
    #[pyfunction]
    fn redirect_stdout(mut message: &str) {
        let mut python_stdout = PYTHON_STDOUT.lock().unwrap();
        while let Some(newline_index) = message.find('\n') {
            info!("python output: {}{}", python_stdout, &message[..newline_index]);
            python_stdout.clear();
            message = &message[newline_index + 1..];
        }
        python_stdout.push_str(message);
    }
    #[pyfunction]
    fn redirect_stderr(mut message: &str) {
        let mut python_stderr = PYTHON_STDERR.lock().unwrap();
        while let Some(newline_index) = message.find('\n') {
            error!("python error: {}{}", python_stderr, &message[..newline_index]);
            python_stderr.clear();
            message = &message[newline_index + 1..];
        }
        python_stderr.push_str(message);
    }

    let m = py.import("sys")?;
    m.add_function(wrap_pyfunction!(redirect_stdout, m)?)?;
    m.add_function(wrap_pyfunction!(redirect_stderr, m)?)?;

    py.run(
        "
import sys
class RedirectStdout:
    def write(message):
        sys.redirect_stdout(message)
    def flush():
        pass
class RedirectStderr:
    def write(message):
        sys.redirect_stderr(message)
    def flush():
        pass
sys.stdout = RedirectStdout
sys.stderr = RedirectStderr"
        , None, None)?;

    Ok(())
}

/// A wrapper around the python interpreter provided by `pyo3`.
pub struct PythonInterpreter;

impl Interpreter for PythonInterpreter {
    fn new() -> TypemakeResult<Self> {
        if PYTHON_INTERPRETER_CREATED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            return Err(TypemakeError::GeneralError("Python interpreter was created more than one time, but supports creation only once".into()));
        }

        // Set up redirection of stdout and stderr through our own logging.
        Python::with_gil(|py| {
            redirect_stdout_stderr(py)
        }).map_err(PythonInterpreterError::from)?;
        Ok(Self)
    }

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
