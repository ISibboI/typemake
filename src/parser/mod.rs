use crate::error::{TypemakeError, TypemakeResult};
use crate::workflow::Tool;
use nom::bytes::complete::{take_till, take_while};
use nom::combinator::{fail, map};
use nom::error::{ParseError, ErrorKind};
use nom::multi::fold_many0;
use nom::{Err};
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::path::Path;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};

#[cfg(test)]
mod tests;

pub type ParserResult<'a, T> = Result<(&'a str, T), nom::Err<ParserError>>;

#[derive(Debug, Eq, Clone, PartialEq, Default)]
pub struct ParserError {
    nom_errors: Vec<(String, ErrorKind)>,
    message: String,
}

impl<'a> Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.nom_errors.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "nom_errors: {:?}, message: {}", self.nom_errors, self.message)
        }
    }
}

impl<'a> ParseError<&'a str> for ParserError {
    fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
        Self {
            nom_errors: vec![(input.to_owned(), kind)],
            message: "".to_string()
        }
    }

    fn append(input: &'a str, kind: ErrorKind, mut other: Self) -> Self {
        other.nom_errors.push((input.to_owned(), kind));
        other
    }
}

impl From<String> for ParserError {
    fn from(message: String) -> Self {
        Self {
            message,
            ..Default::default()
        }
    }
}

impl<'a> From<&'a str> for ParserError {
    fn from(message: &'a str) -> Self {
        Self::from(message.to_owned())
    }
}

#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct Typefile {
    python_lines: Vec<String>,
    tools: BTreeMap<String, Tool>,
}

impl<'a> TryFrom<Vec<ToplevelDefinition>> for Typefile {
    type Error = Err<ParserError>;

    fn try_from(toplevel_definitions: Vec<ToplevelDefinition>) -> Result<Self, Self::Error> {
        let mut result = Self::default();
        for toplevel_definition in toplevel_definitions {
            match toplevel_definition {
                ToplevelDefinition::PythonLine(line) => result.python_lines.push(line),
                ToplevelDefinition::Tool(tool) => {
                    if let Some(tool) = result.tools.insert(tool.name.clone(), tool) {
                        return Err(Err::Failure(ParserError::from(format!("Tool already exists: {:?}", tool.name))));
                    }
                }
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
enum ToplevelDefinition {
    PythonLine(String),
    Tool(Tool),
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

fn nom_typefile(typefile_definition: &str) -> ParserResult<Typefile> {
    let result = fold_many0(
        parse_toplevel_definition,
        Vec::new,
        |mut vec, item| {vec.push(item); vec},
    )(typefile_definition)?;
    if result.0.is_empty() {
        Ok((result.0, result.1.try_into()?))
    } else {
        Err(Err::Failure(ParserError::from("found additional characters after parser terminated")))
    }
}

fn parse_toplevel_definition(s: &str) -> ParserResult<ToplevelDefinition> {
    parse_python_line(s)
}

fn parse_python_line(s: &str) -> ParserResult<ToplevelDefinition> {
    map(take_line, |python_line: &str| {
        ToplevelDefinition::PythonLine(python_line.to_owned())
    })(s)
}

fn take_line(s: &str) -> ParserResult<&str> {
    let line = take_till(|c| c == '\n' || c == '\r')(s)?;
    if line.1.is_empty() {
        return fail(line.0);
    }

    let newline = take_while(|c| c == '\n' || c == '\r')(line.0)?;
    Ok((newline.0, line.1))
}
