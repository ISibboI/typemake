//! The parser for typemake files.

use crate::error::{TypemakeError, TypemakeResult};
use crate::workflow::{Tool, ToolProperty};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::{line_ending, space0, space1, not_line_ending};
use nom::combinator::{fail, iterator, map};
use nom::error::{ErrorKind, ParseError};
use nom::multi::{fold_many0, many0, many1};
use nom::sequence::{pair, tuple};
use nom::{AsChar, Err};
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::path::Path;

#[cfg(test)]
mod tests;

/// The result type used by the parser, that carries the modified input in the `Ok()` variant.
pub type ParserResult<'a, T> = Result<(&'a str, T), nom::Err<ParserError>>;
/// The result type used by the parser, without carrying the modified input.
type ParserResultWithoutInput<T> = Result<T, nom::Err<ParserError>>;

/// The error type of the parser.
/// It collects all nom-errors that occurred,
/// but since these are hard to interpret without looking at the code,
/// `message` gives a human-readable representation of the error.
#[derive(Debug, Eq, Clone, PartialEq, Default)]
pub struct ParserError {
    /// The nom-errors that lead up to this error.
    nom_errors: Vec<(String, ErrorKind)>,
    /// A human-readable representation of the error.
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

/// A parsed typefile.
#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct Typefile {
    /// Everything that is not a structure defined by typemake is collected here line-by-line, to be used as "initialising" code for the typefile.
    pub code_lines: String,
    /// The tool definitions in the typefile.
    pub tools: BTreeMap<String, Tool>,
}

impl<'a> TryFrom<Vec<ToplevelDefinition>> for Typefile {
    type Error = Err<ParserError>;

    fn try_from(toplevel_definitions: Vec<ToplevelDefinition>) -> Result<Self, Self::Error> {
        let mut result = Self::default();
        for toplevel_definition in toplevel_definitions {
            match toplevel_definition {
                ToplevelDefinition::CodeLine(line) => {
                    result.code_lines.push_str(&line);
                    result.code_lines.push('\n')
                }
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

/// A definition on the top level of a typefile.
#[derive(Debug, Clone)]
enum ToplevelDefinition {
    /// A simple line of code without further meaning to typemake.
    CodeLine(String),
    /// A tool definition.
    Tool(Tool),
}

/// Parse the typefile at the given path.
pub fn parse_typefile<P: AsRef<Path> + std::fmt::Debug + Clone>(
    typefile_path: P,
) -> TypemakeResult<Typefile> {
    let typefile_content = read_to_string(typefile_path)?;
    parse_typefile_content(&typefile_content)
}

/// Parse the contents of a typefile (given as `&str`).
pub fn parse_typefile_content(typefile_content: &str) -> TypemakeResult<Typefile> {
    match nom_typefile(typefile_content) {
        Ok((_, result)) => Ok(result),
        Err(err) => Err(TypemakeError::ParserError(err.to_string())),
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
            format!("found additional characters after parser terminated: {:?}", result.0),
        )))
    }
}

/// Parses any definition at the top level of the file, which are all those that don't have any parents.
fn parse_toplevel_definition(s: &str) -> ParserResult<ToplevelDefinition> {
    alt((parse_tool_definition, parse_code_line, parse_empty_line))(s)
}

/// Parses a tool definition, completely with all entries.
/// A tool definition is started by `tool <name>:` and followed by zero or more indented lines with further properties.
fn parse_tool_definition(s: &str) -> ParserResult<ToplevelDefinition> {
    // Parse header
    let (s, header) = tuple((
        tag("tool"),
        space1,
        identifier,
        tag(":"),
        space0,
        many1(line_ending),
    ))(s)?;
    let mut tool = Tool {
        name: header.2.to_owned(),
        ..Default::default()
    };

    // Parse properties.
    // Check for indentation, but do not cut it off.
    // This is to avoid the special case of the first line being unindented in the properties parser.
    let (_, indentation) = space0(s)?;
    let s = if indentation.is_empty() {
        // If the first non-empty line after the header is not indented, the rule has no properties.
        s
    } else {
        let mut tool_property_iterator = iterator(s, parse_tool_property(indentation));
        for tool_property in &mut tool_property_iterator {
            tool_property(&mut tool)?;
        }
        tool_property_iterator.finish()?.0
    };

    // assert unindented line or end of file after tool definition, to ensure that indentation errors are discovered correctly.
    let (s, _) = many0(line_ending)(s)?;
    if !s.is_empty() && (s.starts_with('\t') || s.starts_with(' ')) {
        // Found indented line after end of tool definition.
        return Err(nom::Err::Failure(ParserError::from(format!("Found an indented line after the end of a tool definition. This means that either after the tool definition, there is an indented line that should not be indented, or the indentation of the tool definition is inconsistent."))));
    }

    Ok((s, ToplevelDefinition::Tool(tool)))
}

// fn tool_assigner<'a, PreliminaryType, FinalType>()

/// Parses a property of a tool.
fn parse_tool_property<'indentation, 'result>(
    indentation: &'indentation str,
) -> impl 'result + for<'a> Fn(&'a str) -> ParserResult<'a, ParseToolSetter<'result>>
where
    'indentation: 'result,
{
    move |s: &str| {
        // Skip whitespace-only lines and check for indentation. If there is none, the tool definition is done.
        let (s, _) = pair(many0(pair(space0, many1(line_ending))), tag(indentation))(s)?;

        // Parse specific property.
        alt((
            parse_specific_tool_property("interpreter", indentation, |tool| &mut tool.script),
            fail,
        ))(s)
    }
}

/// A pointer to a function that sets a property value of a tool.
type ParseToolSetter<'a> = Box<dyn 'a + FnOnce(&mut Tool) -> ParserResultWithoutInput<()>>;

/// Parses the interpreter property of a tool.
fn parse_specific_tool_property<
    'indentation,
    'property_name,
    'input,
    'result,
    ToolPropertyPreliminaryType: PartialEq,
    ToolPropertyFinalType: PartialEq,
>(
    property_name: &'property_name str,
    indentation: &'indentation str,
    tool_property_accessor: impl 'result + for<'tool_property_accessor> Fn(&'tool_property_accessor mut Tool) -> &'tool_property_accessor mut ToolProperty<ToolPropertyPreliminaryType, ToolPropertyFinalType> + Clone,
) -> impl 'result + for<'a> FnMut(&'a str) -> ParserResult<'a, ParseToolSetter<'result>>
where
    'property_name: 'result,
    'indentation: 'result,
{
    move |s: &str| {
        // Parse interpreter line.
        let (s, (_, _, _, first_line)) =
            tuple((tag(property_name), tag(":"), space0, take_line_allow_empty))(s)?;
        let mut result = String::from(first_line);

        let s = if let Some(deep_indentation) = check_for_deeper_indentation(s, indentation) {
            // Collect lines that are either whitespace-only or correctly indented.
            let (s, lines) = many0(alt((
                pair(tag(deep_indentation), take_line_disallow_empty),
                map(pair(space0, many1(line_ending)), |_| ("", "")),
            )))(s)?;
            for (_, line) in lines {
                if line.trim().is_empty() {
                    // Skip empty lines.
                    continue;
                }

                result.push('\n');
                result.push_str(line);
            }
            s
        } else {
            s
        };

        let result = String::from(result.trim());
        if result.is_empty() {
            return Err(nom::Err::Failure(ParserError::from(format!(
                "Found an empty-valued interpreter property {:?}.",
                property_name
            ))));
        }

        let tool_property_accessor = tool_property_accessor.clone();
        Ok((
            s,
            Box::new(move |tool| {
                let tool_property = tool_property_accessor(tool);
                if !tool_property.is_empty() {
                    return Err(nom::Err::Failure(ParserError::from(format!(
                        "Found a duplicate definition of {:?} within the same tool.",
                        property_name
                    ))));
                }
                *tool_property = result.into();
                Ok(())
            }),
        ))
    }
}

/// Parse a line as a piece of code.
/// This is the fallback in case the line is of no other type.
fn parse_code_line(s: &str) -> ParserResult<ToplevelDefinition> {
    map(take_line_disallow_empty, |code_line: &str| {
        ToplevelDefinition::CodeLine(code_line.to_owned())
    })(s)
}

/// Parses an empty line.
fn parse_empty_line(s: &str) -> ParserResult<ToplevelDefinition> {
    map(pair(space0, line_ending), |_| {
        ToplevelDefinition::CodeLine("".to_owned())
    })(s)
}

/// Take a full line of output, being robust against different line endings as well as a last line without line ending.
/// If the line taken is empty, return an error.
fn take_line_disallow_empty(s: &str) -> ParserResult<&str> {
    let line = not_line_ending(s)?;
    if line.1.is_empty() {
        return fail(line.0);
    }

    let s = match line_ending::<_, ParserError>(line.0) {
        Ok((s, _)) => s,
        _ => line.0,
    };
    Ok((s, line.1))
}

/// Take a full line of output, being robust against different line endings as well as a last line without line ending.
/// If the line taken is empty, just return an empty `str`.
fn take_line_allow_empty(s: &str) -> ParserResult<&str> {
    let line = not_line_ending(s)?;
    let s = match line_ending::<_, ParserError>(line.0) {
        Ok((s, _)) => s,
        _ => line.0,
    };
    Ok((s, line.1))
}

/// Recognise an identifier.
// Copy-paste from nom::character::complete::alpha1.
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

/// Check if the current line in `s` is indented by at least `shallow_indentation` plus at least one space or tab character.
/// If yes, return the complete indentation of the line, including `shallow_indentation`, if no, return `None`.
fn check_for_deeper_indentation<'input, 'indentation>(
    s: &'input str,
    shallow_indentation: &'indentation str,
) -> Option<&'input str> {
    if let Ok((_, deep_indentation)) = space0::<_, ParserError>(s) {
        if deep_indentation.starts_with(shallow_indentation)
            && deep_indentation.len() > shallow_indentation.len()
        {
            return Some(deep_indentation);
        }
    }

    None
}
