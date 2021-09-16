//! typemake's high-level mode of operation.

use crate::cli::CliArguments;
use crate::error::TypemakeResult;
use crate::interpreter::{Interpreter, SelectedInterpreter};
use crate::parser::parse_typefile;
use log::info;

/// Runs typemake with the given cli-arguments.
/// This is the entrypoint into typemakes business logic.
pub fn run_typemake_from_cli(cli_arguments: &CliArguments) -> TypemakeResult<()> {
    // Parse typefile
    info!("Parsing typefile '{:?}'", &cli_arguments.typefile);
    let workflow = parse_typefile(&cli_arguments.typefile)?;

    info!("Creating interpreter");
    let mut interpreter = SelectedInterpreter::default();
    info!("Interpreter version is {}", interpreter.version()?);

    info!("Executing toplevel scripts");
    interpreter.run(&workflow.code_lines)?;

    info!("Building workflow DAG");

    info!("Terminating");
    Ok(())
}
