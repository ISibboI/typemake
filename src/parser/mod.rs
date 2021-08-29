use crate::error::{TypemakeError, TypemakeResult};
use nom::bytes::complete::{take_till, take_while};
use nom::combinator::{map, fail};
use nom::error::{VerboseError, VerboseErrorKind};
use nom::multi::fold_many0;
use nom::Err;
use nom::IResult;
use std::fs::read_to_string;
use std::path::Path;

#[cfg(test)]
mod tests;

#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct Typefile {
    python_lines: Vec<String>,
}

impl Typefile {
    fn add_toplevel_definition(mut self, toplevel_definition: ToplevelDefinition) -> Self {
        match toplevel_definition {
            ToplevelDefinition::PythonLine(line) => self.python_lines.push(line),
        }
        self
    }
}

#[derive(Debug, Clone)]
enum ToplevelDefinition {
    PythonLine(String),
}

pub fn parse_typefile<P: AsRef<Path> + std::fmt::Debug + Clone>(
    typefile_path: P,
) -> TypemakeResult<Typefile> {
    let typefile_content = read_to_string(typefile_path.clone())?;
    parse_typefile_content(&typefile_content)
}

pub fn parse_typefile_content(typefile_content: &str) -> TypemakeResult<Typefile> {
    match nom_typefile(&typefile_content) {
        Ok((_, result)) => Ok(result),
        Err(err) => Err(TypemakeError::ParseError(err.to_string())),
    }
}

fn nom_typefile(typefile_definition: &str) -> IResult<&str, Typefile, VerboseError<&str>> {
    let result = fold_many0(
        parse_toplevel_definition,
        Typefile::default,
        Typefile::add_toplevel_definition,
    )(typefile_definition)?;
    if result.0.is_empty() {
        Ok(result)
    } else {
        Err(Err::Error(VerboseError {
            errors: vec![(
                result.0,
                VerboseErrorKind::Context("found additional characters after parser terminated"),
            )],
        }))
    }
}

fn parse_toplevel_definition(s: &str) -> IResult<&str, ToplevelDefinition, VerboseError<&str>> {
    parse_python_line(s)
}

fn parse_python_line(s: &str) -> IResult<&str, ToplevelDefinition, VerboseError<&str>> {
    map(take_line, |python_line: &str| {
        ToplevelDefinition::PythonLine(python_line.to_owned())
    })(s)
}

fn take_line(s: &str) -> IResult<&str, &str, VerboseError<&str>> {
    let line = take_till(|c| c == '\n' || c == '\r')(s)?;
    if line.1.is_empty() {
        return fail(line.0);
    }

    let newline = take_while(|c| c == '\n' || c == '\r')(line.0)?;
    Ok((newline.0, line.1))
}
