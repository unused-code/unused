use super::TagProgram;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    combinator::map,
    error::ParseError,
    multi::separated_list,
    sequence::{preceded, terminated, tuple},
    IResult,
};

enum ProgramMetadata {
    Author(String),
    Name(String),
    Version(String),
    Other,
}

impl ProgramMetadata {
    fn author(&self) -> Option<String> {
        match &self {
            ProgramMetadata::Author(v) => Some(v.to_string()),
            _ => None,
        }
    }

    fn name(&self) -> Option<String> {
        match &self {
            ProgramMetadata::Name(v) => Some(v.to_string()),
            _ => None,
        }
    }

    fn version(&self) -> Option<String> {
        match &self {
            ProgramMetadata::Version(v) => Some(v.to_string()),
            _ => None,
        }
    }
}

fn metadata_to_tag_program(metadata: Vec<ProgramMetadata>) -> TagProgram {
    let name = metadata
        .iter()
        .find(|m| m.name().is_some())
        .and_then(|m| m.name());
    let author = metadata
        .iter()
        .find(|m| m.author().is_some())
        .and_then(|m| m.author());
    let version = metadata
        .iter()
        .find(|m| m.version().is_some())
        .and_then(|m| m.version());

    TagProgram {
        name,
        author,
        version,
    }
}

pub fn tag_metadata(input: &str) -> IResult<&str, TagProgram> {
    map(
        terminated(separated_list(tag("\n"), tag_annotation), tag("\n")),
        metadata_to_tag_program,
    )(input)
}

fn tag_annotation(input: &str) -> IResult<&str, ProgramMetadata> {
    alt((program_author, program_name, program_version, program_other))(input)
}

fn tag_value<'a>(tag_name: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, String> {
    move |input| {
        let non_commented_value = preceded(terminated(tag(tag_name), tag("\t")), to_tab);
        map(
            tuple((non_commented_value, metadata_comment)),
            |(value, comment)| format!("{}{}", value, parenthetical(comment)),
        )(input)
    }
}

fn parenthetical(value: Option<String>) -> String {
    match value {
        Some(v) => format!(" ({})", v),
        None => "".to_string(),
    }
}

fn optional_string(input: &str) -> Option<&str> {
    match input.trim() {
        "" => None,
        v => Some(v),
    }
}

fn metadata_comment(input: &str) -> IResult<&str, Option<String>> {
    let (input, _) = tag("/")(input)?;
    map(terminated(take_till(|c| c == '/'), tag("/")), |v| {
        optional_string(v).map(String::from)
    })(input)
}

fn program_other(input: &str) -> IResult<&str, ProgramMetadata> {
    map(preceded(tag("!_TAG"), to_newline), |_| {
        ProgramMetadata::Other
    })(input)
}

fn program_author(input: &str) -> IResult<&str, ProgramMetadata> {
    map(tag_value("!_TAG_PROGRAM_AUTHOR"), ProgramMetadata::Author)(input)
}

fn program_name(input: &str) -> IResult<&str, ProgramMetadata> {
    map(tag_value("!_TAG_PROGRAM_NAME"), ProgramMetadata::Name)(input)
}

fn program_version(input: &str) -> IResult<&str, ProgramMetadata> {
    map(tag_value("!_TAG_PROGRAM_VERSION"), ProgramMetadata::Version)(input)
}

pub fn succeed<I: Clone, O, F: Copy + FnOnce() -> O, E: ParseError<I>>(
    success: F,
) -> impl Fn(I) -> IResult<I, O, E> {
    move |input: I| Ok((input, success()))
}

pub fn to_tab(input: &str) -> IResult<&str, &str> {
    terminated(take_till(|c| c == '\t'), tag("\t"))(input)
}

pub fn to_newline(input: &str) -> IResult<&str, &str> {
    take_till(|c| c == '\n')(input)
}

#[test]
fn parses_metadata_comment() {
    assert_eq!(
        metadata_comment("/dhiebert@users.sourceforge.net/"),
        Ok(("", Some("dhiebert@users.sourceforge.net".to_string())))
    );
}

#[test]
fn parses_tag_value() {
    assert_eq!(
        tag_value("!_TAG_PROGRAM_AUTHOR")(
            "!_TAG_PROGRAM_AUTHOR	Darren Hiebert	/dhiebert@users.sourceforge.net/"
        ),
        Ok((
            "",
            "Darren Hiebert (dhiebert@users.sourceforge.net)".to_string()
        ))
    );
}
