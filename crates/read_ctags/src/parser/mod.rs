pub mod internal;
use super::ctag_item::CtagItem;
use super::language::Language;
use super::token_kind::{calculate_kind, TokenKind};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{alphanumeric1, anychar},
    combinator::{map, verify},
    error::context,
    multi::separated_list,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParsedField<'a> {
    KindField(char),
    ParsedField(&'a str, &'a str),
}

pub fn parse(input: &str) -> IResult<&str, Vec<CtagItem>> {
    let (input, _) = internal::tag_metadata(input)?;
    terminated(separated_list(tag("\n"), ctag_item_parser), tag("\n"))(input)
}

fn is_kind(field: &ParsedField) -> bool {
    match field {
        ParsedField::KindField(_) => true,
        _ => false,
    }
}

fn key_value_parser(input: &str) -> IResult<&str, ParsedField> {
    map(
        separated_pair(
            alphanumeric1,
            tag(":"),
            take_while(|c| c != '\t' && c != '\n'),
        ),
        |(k, v)| ParsedField::ParsedField(k, v),
    )(input)
}

fn kind_parser(input: &str) -> IResult<&str, ParsedField> {
    map(anychar, ParsedField::KindField)(input)
}

fn fields_parser(input: &str) -> IResult<&str, Vec<ParsedField>> {
    let field_parser = alt((key_value_parser, kind_parser));
    preceded(tag("\t"), separated_list(tag("\t"), field_parser))(input)
}

fn address_and_fields_parser(input: &str) -> IResult<&str, (&str, Vec<ParsedField>)> {
    alt((
        tuple((tag_address_parser, fields_parser)),
        tuple((
            tag_address_without_fields_parser,
            internal::succeed(|| vec![]),
        )),
    ))(input)
}

fn tag_address_parser(input: &str) -> IResult<&str, &str> {
    terminated(
        verify(take_until(";\""), |s: &str| !s.contains('\n')),
        tag(";\""),
    )(input)
}

fn tag_address_without_fields_parser(input: &str) -> IResult<&str, &str> {
    internal::to_newline(input)
}

fn ctag_item_parser(input: &str) -> IResult<&str, CtagItem> {
    let (input, name) = context("tagName", internal::to_tab)(input)?;
    let (input, file_path) = context("tagPath", internal::to_tab)(input)?;
    let (input, (_, parsed_fields)) = address_and_fields_parser(input)?;
    let language = Language::from_path(file_path);
    let (kind, tags) = build_kind_and_fields(language, parsed_fields);

    Ok((
        input,
        CtagItem {
            name: name.to_string(),
            file_path: file_path.to_string(),
            language,
            tags,
            kind,
        },
    ))
}

fn build_kind_and_fields<'a>(
    language: Option<Language>,
    parsed_fields: Vec<ParsedField<'a>>,
) -> (TokenKind, BTreeMap<String, String>) {
    let (kind, rest): (Vec<ParsedField>, Vec<ParsedField>) =
        parsed_fields.iter().partition(|&f| is_kind(f));

    let mut hash = BTreeMap::new();

    for field in rest.iter() {
        match field {
            ParsedField::ParsedField(k, v) => hash.insert((*k).to_string(), (*v).to_string()),
            _ => None,
        };
    }

    match (kind.len(), kind.get(0)) {
        (1, Some(ParsedField::KindField(c))) => (calculate_kind(language, *c), hash),
        (_, _) => (TokenKind::Undefined, hash),
    }
}

#[test]
fn parses_fields() {
    assert_eq!(
        key_value_parser("foo:bar"),
        Ok(("", ParsedField::ParsedField("foo", "bar")))
    );
}

#[test]
fn parses_item_lines() {
    assert_eq!(
        ctag_item_parser("withInfo\tpath/to/file.rb\t45"),
        Ok((
            "",
            CtagItem {
                name: String::from("withInfo"),
                file_path: String::from("path/to/file.rb"),
                language: Some(Language::Ruby),
                tags: BTreeMap::new(),
                kind: TokenKind::Undefined
            }
        ))
    );
}

#[test]
fn parses_multiple_lines() {
    assert_eq!(
        parse("!_TAG_INFO\nfirst\tpath/to/file.rb\t1\nsecond\tpath/to/file.rb\t2;\"\tc\n"),
        Ok((
            "",
            vec![
                CtagItem {
                    name: String::from("first"),
                    file_path: String::from("path/to/file.rb"),
                    language: Some(Language::Ruby),
                    tags: BTreeMap::new(),
                    kind: TokenKind::Undefined
                },
                CtagItem {
                    name: String::from("second"),
                    file_path: String::from("path/to/file.rb"),
                    language: Some(Language::Ruby),
                    tags: BTreeMap::new(),
                    kind: TokenKind::Class
                }
            ]
        ))
    );
}

#[test]
fn parses_multiple_fields() {
    assert_eq!(
        fields_parser("\tfoo:bar\ta\tbaz:buzz"),
        Ok((
            "",
            vec![
                ParsedField::ParsedField("foo", "bar"),
                ParsedField::KindField('a'),
                ParsedField::ParsedField("baz", "buzz"),
            ]
        ))
    );
}

#[test]
fn parses_fields_with_non_alphanumeric_values() {
    assert_eq!(
        fields_parser("\tarray:app/jobs/*.rb.template"),
        Ok((
            "",
            vec![ParsedField::ParsedField("array", "app/jobs/*.rb.template"),]
        ))
    );
}

#[test]
fn parses_kinds_only() {
    assert_eq!(
        fields_parser("\tv"),
        Ok(("", vec![ParsedField::KindField('v')]))
    );
}

#[test]
fn parses_addresses_with_fields() {
    assert_eq!(
        address_and_fields_parser("/^  context \"#active\" do$/;\"\tc"),
        Ok((
            "",
            (
                "/^  context \"#active\" do$/",
                vec![ParsedField::KindField('c'),]
            )
        ))
    );
}

#[test]
fn parses_when_address_includes_semicolon() {
    assert_eq!(
        address_and_fields_parser("/^$z-tooltip: $base-z-index + 18;$/;\"\tv"),
        Ok((
            "",
            (
                "/^$z-tooltip: $base-z-index + 18;$/",
                vec![ParsedField::KindField('v'),]
            )
        ))
    );
}
