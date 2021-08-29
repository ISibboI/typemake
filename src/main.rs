use log::LevelFilter;
use simplelog::{ColorChoice, TermLogger, TerminalMode};
use log::{error};
use error_chain::error_chain;
use crate::typemake::run_typemake_from_cli;
use cli::CliArguments;
use clap::Clap;

mod parser;
mod cli;
mod typemake;

error_chain! {
    types {
        TypemakeError, TypemakeErrorKind, TypemakeResultExt, TypemakeResult;
    }

    links {
        ParserError(parser::ParserError, parser::ParserErrorKind);
    }

    errors {

    }
}

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