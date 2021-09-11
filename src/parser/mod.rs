use crate::error::{TypemakeError, TypemakeResult};
use crate::workflow::Tool;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::{take_till, take_while};
use nom::character::complete::{line_ending, space0, space1};
use nom::combinator::{fail, map};
use nom::error::{ErrorKind, ParseError};
use nom::multi::{fold_many0, fold_many1, many0, many1};
use nom::sequence::{pair, tuple};
use nom::{AsChar, Err, InputLength, Parser};
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
        //many0(parse_tool_property(indentation))((s, tool))?.0.0
        ref_mut_many0(parse_tool_property(indentation), &mut tool)(s)?.0
    };

    Ok((s, ToplevelDefinition::Tool(tool)))
}

/// Parses a property of a tool.
fn parse_tool_property<'a>(
    indentation: &'a str,
) -> impl for<'tool> Fn(&'a str, &'tool mut Tool) -> ParserResult<'a, ()> {
    move |s: &'a str, tool: &mut Tool| {
        // Skip empty lines and check for indentation. If there is none, the tool definition is done.
        let (s, _) = pair(many0(line_ending), tag(indentation))(s)?;

        // Parse specific property.
        let (s, _) = ref_mut_alt((parse_tool_property_script(indentation), fail), tool)(s)?;
        Ok((s, ()))
    }
}

/// Parses the script property of a tool.
fn parse_tool_property_script<'a>(indentation: &'a str) -> impl for<'tool> FnMut(&'a str, &'tool mut Tool) -> ParserResult<'a, ()> {
    move |s: &'a str, tool: &mut Tool| {
        // Parse script line.
        let (s, result) = tuple((tag("script:"), space0, take_line))(s)?;
        tool.script = Some(result.2.to_owned());

        Ok((s, ()))
    }
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

/// Applies a parser until it fails while passing through a value.
/// TODO
pub fn pass_through_many0<Input, Output, IntermediateOutput, Error, ParserType>(
    mut parser: ParserType,
    initial_value: Output,
) -> impl FnOnce(Input) -> nom::IResult<Input, Output, Error>
where
    Input: Clone + InputLength,
    Error: ParseError<Input>,
    ParserType: FnMut(Input, Output) -> (nom::IResult<Input, IntermediateOutput, Error>, Output),
{
    move |mut input: Input| {
        let mut current_value = initial_value;

        loop {
            match parser(input.clone(), current_value) {
                (Ok((processed_input, _)), processed_value) => {
                    // infinite loop check: the parser must always consume
                    if processed_input.input_len() == input.input_len() {
                        return Err(Err::Error(Error::from_error_kind(input, ErrorKind::Many0)));
                    }

                    input = processed_input;
                    current_value = processed_value;
                }
                (Err(Err::Error(_)), processed_value) => {
                    return Ok((input, processed_value));
                }
                (Err(e), _) => {
                    return Err(e);
                }
            }
        }
    }
}

/// Applies a parser until it fails while passing it a mutable reference.
/// TODO
pub fn ref_mut_many0<'output, Input, Output, IntermediateOutput, Error, ParserType>(
    mut parser: ParserType,
    output: &'output mut Output,
) -> impl FnOnce(Input) -> nom::IResult<Input, &'output mut Output, Error>
where
    Input: Clone + InputLength,
    Error: ParseError<Input>,
    ParserType:
        for<'a> FnMut(Input, &'a mut Output) -> nom::IResult<Input, IntermediateOutput, Error>,
{
    move |mut input: Input| {
        loop {
            match parser(input.clone(), output) {
                Ok((processed_input, _)) => {
                    // infinite loop check: the parser must always consume
                    if processed_input.input_len() == input.input_len() {
                        return Err(Err::Error(Error::from_error_kind(input, ErrorKind::Many0)));
                    }

                    input = processed_input;
                }
                Err(Err::Error(_)) => {
                    return Ok((input, output));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

/// Tests a list of parsers one by one until one succeeds.
/// It passes a mutable reference to each parser.
/// TODO
pub fn ref_mut_alt<
    'output,
    Input: Clone,
    Output,
    IntermediateOutput,
    Error: ParseError<Input>,
    List: for<'a> RefMutAlt<Input, &'a mut Output, IntermediateOutput, Error>,
>(
    mut l: List,
    output: &'output mut Output,
) -> impl FnOnce(Input) -> nom::IResult<Input, &'output mut Output, Error> {
    move |s: Input| {
        let ((s, _), _) = l.ref_mut_choice(s, output)?;
        Ok((s, output))
    }
}


// TODO make RefMutParser trait to implement this more easily
pub trait RefMutAlt<
    Input, Output, IntermediateOutput, Error
> {
    fn ref_mut_choice(&mut self, input: Input, output: &mut Output) -> nom::IResult<Input, IntermediateOutput, Error>;
}

macro_rules! succ (
  (0, $submac:ident ! ($($rest:tt)*)) => ($submac!(1, $($rest)*));
  (1, $submac:ident ! ($($rest:tt)*)) => ($submac!(2, $($rest)*));
  (2, $submac:ident ! ($($rest:tt)*)) => ($submac!(3, $($rest)*));
  (3, $submac:ident ! ($($rest:tt)*)) => ($submac!(4, $($rest)*));
  (4, $submac:ident ! ($($rest:tt)*)) => ($submac!(5, $($rest)*));
  (5, $submac:ident ! ($($rest:tt)*)) => ($submac!(6, $($rest)*));
  (6, $submac:ident ! ($($rest:tt)*)) => ($submac!(7, $($rest)*));
  (7, $submac:ident ! ($($rest:tt)*)) => ($submac!(8, $($rest)*));
  (8, $submac:ident ! ($($rest:tt)*)) => ($submac!(9, $($rest)*));
  (9, $submac:ident ! ($($rest:tt)*)) => ($submac!(10, $($rest)*));
  (10, $submac:ident ! ($($rest:tt)*)) => ($submac!(11, $($rest)*));
  (11, $submac:ident ! ($($rest:tt)*)) => ($submac!(12, $($rest)*));
  (12, $submac:ident ! ($($rest:tt)*)) => ($submac!(13, $($rest)*));
  (13, $submac:ident ! ($($rest:tt)*)) => ($submac!(14, $($rest)*));
  (14, $submac:ident ! ($($rest:tt)*)) => ($submac!(15, $($rest)*));
  (15, $submac:ident ! ($($rest:tt)*)) => ($submac!(16, $($rest)*));
  (16, $submac:ident ! ($($rest:tt)*)) => ($submac!(17, $($rest)*));
  (17, $submac:ident ! ($($rest:tt)*)) => ($submac!(18, $($rest)*));
  (18, $submac:ident ! ($($rest:tt)*)) => ($submac!(19, $($rest)*));
  (19, $submac:ident ! ($($rest:tt)*)) => ($submac!(20, $($rest)*));
  (20, $submac:ident ! ($($rest:tt)*)) => ($submac!(21, $($rest)*));
);

macro_rules! ref_mut_alt_trait(
  ($first:ident $second:ident $($id: ident)+) => (
    ref_mut_alt_trait!(__impl $first $second; $($id)+);
  );
  (__impl $($current:ident)*; $head:ident $($id: ident)+) => (
    ref_mut_alt_trait_impl!($($current)*);

    ref_mut_alt_trait!(__impl $($current)* $head; $($id)+);
  );
  (__impl $($current:ident)*; $head:ident) => (
    ref_mut_alt_trait_impl!($($current)*);
    ref_mut_alt_trait_impl!($($current)* $head);
  );
);

macro_rules! ref_mut_alt_trait_impl(
  ($($id:ident)+) => (
    impl<
      Input: Clone, Output, IntermediateOutput, Error: ParseError<Input>,
      $($id: Parser<Input, Output, Error>),+
    > RefMutAlt<Input, Output, IntermediateOutput, Error> for ( $($id),+ ) {

      fn ref_mut_choice(&mut self, input: Input, output: &mut Output) -> IResult<Input, IntermediateOutput, Error> {
        match self.0.parse(input.clone()) {
          Err(Err::Error(e)) => ref_mut_alt_trait_inner!(1, self, input, output, e, $($id)+),
          res => res,
        }
      }
    }
  );
);

macro_rules! ref_mut_alt_trait_inner(
  ($it:tt, $self:expr, $input:expr, $output:expr, $err:expr, $head:ident $($id:ident)+) => (
    match $self.$it.parse($input.clone()) {
      Err(Err::Error(e)) => {
        let err = $err.or(e);
        succ!($it, ref_mut_alt_trait_inner!($self, $input, err, $($id)+))
      }
      res => res,
    }
  );
  ($it:tt, $self:expr, $input:expr, $err:expr, $head:ident) => (
    Err(Err::Error(Error::append($input, ErrorKind::Alt, $err)))
  );
);

ref_mut_alt_trait!(A B C D E F G H I J K L M N O P Q R S T U);