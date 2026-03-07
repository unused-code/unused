use super::TagProgram;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till},
    combinator::map,
    error::ParseError,
    multi::separated_list0,
    sequence::{preceded, terminated},
};

enum ProgramMetadata {
    Author(String),
    Name(String),
    Version(String),
    Other,
}

fn metadata_to_tag_program(metadata: Vec<ProgramMetadata>) -> TagProgram {
    let mut name = None;
    let mut author = None;
    let mut version = None;

    for item in metadata {
        match item {
            ProgramMetadata::Name(v) if name.is_none() => name = Some(v),
            ProgramMetadata::Author(v) if author.is_none() => author = Some(v),
            ProgramMetadata::Version(v) if version.is_none() => version = Some(v),
            _ => {}
        }
    }

    TagProgram {
        name,
        author,
        version,
    }
}

pub fn tag_metadata(input: &str) -> IResult<&str, TagProgram> {
    map(
        terminated(separated_list0(tag("\n"), tag_annotation), tag("\n")),
        metadata_to_tag_program,
    )
    .parse(input)
}

fn tag_annotation(input: &str) -> IResult<&str, ProgramMetadata> {
    alt((program_author, program_name, program_version, program_other)).parse(input)
}

fn tag_value<'a>(tag_name: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, String> {
    move |input| {
        let non_commented_value = preceded(terminated(tag(tag_name), tag("\t")), to_tab);
        map(
            (non_commented_value, metadata_comment),
            |(value, comment)| format!("{}{}", value, parenthetical(comment)),
        )
        .parse(input)
    }
}

fn parenthetical(value: Option<&str>) -> String {
    match value {
        Some(v) => format!(" ({v})"),
        None => String::new(),
    }
}

fn optional_string(input: &str) -> Option<&str> {
    match input.trim() {
        "" => None,
        v => Some(v),
    }
}

fn metadata_comment(input: &str) -> IResult<&str, Option<&str>> {
    let (input, _) = tag("/").parse(input)?;
    map(
        terminated(take_till(|c| c == '/'), tag("/")),
        optional_string,
    )
    .parse(input)
}

fn program_other(input: &str) -> IResult<&str, ProgramMetadata> {
    map(preceded(tag("!_TAG"), to_newline), |_| {
        ProgramMetadata::Other
    })
    .parse(input)
}

fn program_author(input: &str) -> IResult<&str, ProgramMetadata> {
    map(tag_value("!_TAG_PROGRAM_AUTHOR"), ProgramMetadata::Author).parse(input)
}

fn program_name(input: &str) -> IResult<&str, ProgramMetadata> {
    map(tag_value("!_TAG_PROGRAM_NAME"), ProgramMetadata::Name).parse(input)
}

fn program_version(input: &str) -> IResult<&str, ProgramMetadata> {
    map(tag_value("!_TAG_PROGRAM_VERSION"), ProgramMetadata::Version).parse(input)
}

pub fn succeed<I: Clone, O, F: Copy + FnOnce() -> O, E: ParseError<I>>(
    success: F,
) -> impl Fn(I) -> IResult<I, O, E> {
    move |input: I| Ok((input, success()))
}

pub fn to_tab(input: &str) -> IResult<&str, &str> {
    terminated(take_till(|c| c == '\t'), tag("\t")).parse(input)
}

pub fn to_newline(input: &str) -> IResult<&str, &str> {
    take_till(|c| c == '\n').parse(input)
}

#[test]
fn parses_metadata_comment() {
    assert_eq!(
        metadata_comment("/dhiebert@users.sourceforge.net/"),
        Ok(("", Some("dhiebert@users.sourceforge.net")))
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
