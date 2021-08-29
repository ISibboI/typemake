use std::path::Path;
use std::fs::read_to_string;
use error_chain::{error_chain, ensure};
use nom::{IResult};
use nom::error::{VerboseError};

#[cfg(test)]
mod tests;

error_chain!{
    types {
        ParserError, ParserErrorKind, ParserResultExt, ParserResult;
    }

    foreign_links {
        Io(::std::io::Error);
        //NomError(nom::error::Error);
    }

    errors {
        NomError(error: String) {
            description("error parsing typefile")
            display("error parsing typefile: {}", error)
        }
    }
}

#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct Typefile {
    _python_lines: Vec<String>,
}

pub fn parse_typefile<P: AsRef<Path> + std::fmt::Debug + Clone>(typefile_path: P) -> ParserResult<Typefile> {
    let typefile_definition = read_to_string(typefile_path.clone()).chain_err(|| format!("Could not access typefile {:?}", typefile_path))?;
    match nom_typefile(&typefile_definition) {
        Ok((remaining_typefile_definition, typefile)) => {
            ensure!(remaining_typefile_definition.is_empty(), "Parsing was successful, but there are unparsed characters: '{}'", remaining_typefile_definition);
            Ok(typefile)
        }
        Err(err) => {
            Err(ParserErrorKind::NomError(err.to_string()).into())
        }
    }
}

fn nom_typefile(typefile_definition: &str) -> IResult<&str, Typefile, VerboseError<&str>> {
    let typefile = Typefile::default();
    Ok((typefile_definition, typefile))
}