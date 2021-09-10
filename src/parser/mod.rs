use crate::error::{TypemakeError, TypemakeResult};
use crate::workflow::Tool;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::{take_till, take_while};
use nom::character::complete::{line_ending, space0, space1};
use nom::combinator::{fail, map};
use nom::error::{ErrorKind, ParseError};
use nom::multi::fold_many0;
use nom::sequence::tuple;
use nom::{AsChar, Err};
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::path::Path;

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
            write!(
                f,
                "nom_errors: {:?}, message: {}",
                self.nom_errors, self.message
            )
        }
    }
}

impl<'a> ParseError<&'a str> for ParserError {
    fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
        Self {
            nom_errors: vec![(input.to_owned(), kind)],
            message: "".to_string(),
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
    code_lines: Vec<String>,
    tools: BTreeMap<String, Tool>,
}

impl<'a> TryFrom<Vec<ToplevelDefinition>> for Typefile {
    type Error = Err<ParserError>;

    fn try_from(toplevel_definitions: Vec<ToplevelDefinition>) -> Result<Self, Self::Error> {
        let mut result = Self::default();
        for toplevel_definition in toplevel_definitions {
            match toplevel_definition {
                ToplevelDefinition::CodeLine(line) => result.code_lines.push(line),
                ToplevelDefinition::Tool(tool) => {
                    if let Some(tool) = result.tools.insert(tool.name.clone(), tool) {
                        return Err(Err::Failure(ParserError::from(format!(
                            "Tool already exists: {:?}",
                            tool.name
                        ))));
                    }
                }
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
enum ToplevelDefinition {
    CodeLine(String),
    Tool(Tool),
}

pub fn parse_typefile<P: AsRef<Path> + std::fmt::Debug + Clone>(
    typefile_path: P,
) -> TypemakeResult<Typefile> {
    let typefile_content = read_to_string(typefile_path)?;
    parse_typefile_content(&typefile_content)
}

pub fn parse_typefile_content(typefile_content: &str) -> TypemakeResult<Typefile> {
    match nom_typefile(typefile_content) {
        Ok((_, result)) => Ok(result),
        Err(err) => Err(TypemakeError::ParseError(err.to_string())),
    }
}

/// Parse a whole typefile.
/// This is the root of the nom-part of the parser.
fn nom_typefile(typefile_definition: &str) -> ParserResult<Typefile> {
    let result = fold_many0(parse_toplevel_definition, Vec::new, |mut vec, item| {
        vec.push(item);
        vec
    })(typefile_definition)?;
    if result.0.is_empty() {
        Ok((result.0, result.1.try_into()?))
    } else {
        Err(Err::Failure(ParserError::from(
            "found additional characters after parser terminated",
        )))
    }
}

/// Parses any definition at the top level of the file, which are all those that don't have any parents.
fn parse_toplevel_definition(s: &str) -> ParserResult<ToplevelDefinition> {
    alt((parse_tool_definition, parse_code_line))(s)
}

fn parse_tool_definition(s: &str) -> ParserResult<ToplevelDefinition> {
    let result = tuple((
        tag("tool"),
        space1,
        identifier,
        tag(":"),
        space0,
        line_ending,
    ))(s)?;
    Ok((
        result.0,
        ToplevelDefinition::Tool(Tool {
            name: result.1 .2.to_owned(),
        }),
    ))
}

/// Parse a line as a piece of code.
/// This is the fallback in case the line is of no other type.
fn parse_code_line(s: &str) -> ParserResult<ToplevelDefinition> {
    map(take_line, |code_line: &str| {
        ToplevelDefinition::CodeLine(code_line.to_owned())
    })(s)
}

/// Take a full line of output, being robust against different line endings as well as a last line without line ending.
fn take_line(s: &str) -> ParserResult<&str> {
    let line = take_till(|c| c == '\n' || c == '\r')(s)?;
    if line.1.is_empty() {
        return fail(line.0);
    }

    let newline = take_while(|c| c == '\n' || c == '\r')(line.0)?;
    Ok((newline.0, line.1))
}

/// Recognise an identifier.
// Copy-paste from nom::character::complete::alpha1
fn identifier<T, E: ParseError<T>>(input: T) -> nom::IResult<T, T, E>
where
    T: nom::InputTakeAtPosition,
    <T as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
{
    input.split_at_position1_complete(
        |item| !item.clone().is_alpha() && item.as_char() != '_',
        ErrorKind::Alpha,
    )
}
