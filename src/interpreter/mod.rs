//! The interpreter used to evaluate scripts in the typemake file.

use crate::error::TypemakeResult;

#[cfg(feature = "python")]
mod python;

/// The interpreter selected by the compilation configuration.
#[cfg(feature = "python")]
pub type SelectedInterpreter = python::PythonInterpreter;
/// The error type of the interpreter selected by the compilation configuration.
#[cfg(feature = "python")]
pub type InterpreterError = python::PythonInterpreterError;

/// An interpreter for scripts given in the typefile.
pub trait Interpreter: Sized {
    /// Creates a new interpreter instance.
    /// Interpreters might be limited to a single instance.
    fn new() -> TypemakeResult<Self>;

    /// Runs the given code while
    fn run(&mut self, script: &str) -> TypemakeResult<()>;

    /// Returns the version
    fn version(&self) -> TypemakeResult<String>;
}
