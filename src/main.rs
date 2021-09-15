#![allow(clippy::useless_format)]

use crate::error::TypemakeResult;
use crate::typemake::run_typemake_from_cli;
use clap::Clap;
use cli::CliArguments;
use log::error;
use log::LevelFilter;
use simplelog::{ColorChoice, TermLogger, TerminalMode};

mod cli;
mod error;
mod parser;
mod typemake;
mod workflow;

fn main() {
    if let Err(error) = error_main() {
        error!("Error:\n{}", error.to_string());
    }
}

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
