//! A workflow engine designed for dynamic, efficient, reproducible and iterative experiment design.

#![allow(clippy::useless_format)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use crate::error::TypemakeResult;
use crate::typemake::run_typemake_from_cli;
use clap::Clap;
use cli::CliArguments;
use log::error;
use log::LevelFilter;
use simplelog::{ColorChoice, TermLogger, TerminalMode};

mod cli;
mod error;
mod interpreter;
mod parser;
mod typemake;
mod workflow;

/// Helper main function that executes the actual main function (`error_main`) and formats any errors it returns.
fn main() -> TypemakeResult<()> {
    let result = error_main();
    if let Err(error) = &result {
        error!("Error:\n{}", error.to_string());
    }
    result
}

/// The actual main function that is allowed to return an error, which is then properly formatted by the `main` function.
fn error_main() -> TypemakeResult<()> {
    // Init logging
    TermLogger::init(
        LevelFilter::Trace,
        Default::default(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    )
    .expect("Could not initialize logging");

    // Parse cli arguments
    let cli_arguments = CliArguments::parse();

    // Run typemake
    run_typemake_from_cli(&cli_arguments)
}
