use crate::cli::CliArguments;
use crate::error::TypemakeResult;
use crate::parser::parse_typefile;
use log::info;

pub fn run_typemake_from_cli(cli_arguments: &CliArguments) -> TypemakeResult<()> {
    // Parse typefile
    info!("Parsing typefile '{:?}'", &cli_arguments.typefile);
    let _workflow = parse_typefile(&cli_arguments.typefile)?;

    info!("Terminating");
    Ok(())
}
