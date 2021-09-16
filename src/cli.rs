//! The command line interface of typemake.

use clap::Clap;
use std::path::PathBuf;

/// The command line arguments of a typemake.
#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = "Sebastian Schmidt <isibboi@gmail.com>")]
pub struct CliArguments {
    #[clap(
        long,
        name = "typefile",
        about = "The root typefile to execute",
        default_value = "Typefile"
    )]
    /// The path to the root typefile.
    pub typefile: PathBuf,
}
