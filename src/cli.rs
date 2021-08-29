use std::path::PathBuf;
use clap::Clap;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = "Sebastian Schmidt <isibboi@gmail.com>")]
pub struct CliArguments {
    #[clap(
    name = "typefile",
    index = 1,
    about = "The root typefile to execute",
    default_value = "Typefile"
    )]
    pub typefile: PathBuf,
}