use convert_case::{Case, Casing};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till, take_till1},
    character::complete::{char, space0},
    combinator::{all_consuming, recognize, value},
    multi::{many0, separated_list1},
    sequence::{delimited, pair, separated_pair},
};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct RawAliasRule {
    from: String,
    to: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AliasRule {
    from: AliasFromPattern,
    to: AliasTemplate,
}

impl AliasRule {
    #[allow(dead_code)]
    pub(crate) fn new(from: AliasFromPattern, to: AliasTemplate) -> Self {
        Self { from, to }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AliasFromPattern {
    pub raw: String,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AliasTemplate {
    pub raw: String,
    pub parts: Vec<AliasTemplatePart>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AliasTransform {
    Snakecase,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AliasTemplatePart {
    Literal(String),
    Capture(Vec<AliasTransform>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AliasRuleField {
    From,
    To,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct AliasRuleValidationError {
    pub rule_index: usize,
    pub field: AliasRuleField,
    pub message: String,
}

impl AliasRuleValidationError {
    fn from_error(rule_index: usize, message: String) -> Self {
        Self {
            rule_index,
            field: AliasRuleField::From,
            message,
        }
    }

    fn to_error(rule_index: usize, message: String) -> Self {
        Self {
            rule_index,
            field: AliasRuleField::To,
            message,
        }
    }
}

/// Compile raw alias rules into parsed matching/rendering components.
///
/// # Errors
///
/// Returns all rule validation errors when one or more `from`/`to` patterns are invalid.
pub fn compile_alias_rules(
    raw_rules: &[RawAliasRule],
) -> Result<Vec<AliasRule>, Vec<AliasRuleValidationError>> {
    let mut compiled_rules = Vec::new();
    let mut errors = Vec::new();

    for (rule_index, raw_rule) in raw_rules.iter().enumerate() {
        let from = match parse_from_pattern(rule_index, &raw_rule.from) {
            Ok(from) => from,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };

        let to = match parse_template(rule_index, &raw_rule.to) {
            Ok(to) => to,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };

        compiled_rules.push(AliasRule { from, to });
    }

    if errors.is_empty() {
        Ok(compiled_rules)
    } else {
        Err(errors)
    }
}

#[must_use]
pub fn expand_alias_candidates(input: &str, alias_rules: &[AliasRule]) -> HashSet<String> {
    let mut candidates = HashSet::from([input.to_string()]);

    for alias_rule in alias_rules {
        let Some(capture) = input
            .strip_prefix(&alias_rule.from.prefix)
            .and_then(|rest| rest.strip_suffix(&alias_rule.from.suffix))
        else {
            continue;
        };

        let generated = render_alias_template(capture, &alias_rule.to.parts);
        if generated != input {
            candidates.insert(generated);
        }
    }

    candidates
}

#[must_use]
pub fn render_alias_template(capture: &str, parts: &[AliasTemplatePart]) -> String {
    let mut rendered = String::new();

    for part in parts {
        match part {
            AliasTemplatePart::Literal(value) => rendered.push_str(value),
            AliasTemplatePart::Capture(transforms) => {
                rendered.push_str(&apply_alias_transforms(capture, transforms));
            }
        }
    }

    rendered
}

#[must_use]
pub fn apply_alias_transforms(value: &str, transforms: &[AliasTransform]) -> String {
    transforms
        .iter()
        .fold(value.to_string(), |acc, transform| transform.apply(&acc))
}

impl AliasTransform {
    fn apply(&self, value: &str) -> String {
        match self {
            AliasTransform::Snakecase => value.to_case(Case::Snake),
        }
    }
}

fn parse_from_pattern(
    rule_index: usize,
    raw_pattern: &str,
) -> Result<AliasFromPattern, AliasRuleValidationError> {
    let (_, (prefix, suffix)) = alias_from_pattern(raw_pattern).map_err(|_| {
        AliasRuleValidationError::from_error(
            rule_index,
            "from pattern must contain exactly one `*` wildcard".to_string(),
        )
    })?;

    if suffix.contains('*') {
        return Err(AliasRuleValidationError::from_error(
            rule_index,
            "from pattern must contain exactly one `*` wildcard".to_string(),
        ));
    }

    Ok(AliasFromPattern {
        raw: raw_pattern.to_string(),
        prefix: prefix.to_string(),
        suffix: suffix.to_string(),
    })
}

fn alias_from_pattern(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(take_till(|ch| ch == '*'), char('*'), take_till(|_| false)).parse(input)
}

fn literal_chunk(input: &str) -> IResult<&str, &str> {
    take_till1(|ch| ch == '{').parse(input)
}

fn parse_template(
    rule_index: usize,
    raw_template: &str,
) -> Result<AliasTemplate, AliasRuleValidationError> {
    let mut parts = Vec::new();
    let mut remaining = raw_template;

    while !remaining.is_empty() {
        if remaining.starts_with('{') {
            let (rest, capture_token) = capture_token_raw(remaining).map_err(|_| {
                AliasRuleValidationError::to_error(
                    rule_index,
                    "to template has an unclosed `{` token".to_string(),
                )
            })?;

            let transforms = parse_capture_transforms(rule_index, capture_token)?;
            parts.push(AliasTemplatePart::Capture(transforms));
            remaining = rest;
            continue;
        }

        let (rest, literal) = literal_chunk(remaining).map_err(|_| {
            AliasRuleValidationError::to_error(rule_index, "invalid to template".to_string())
        })?;
        parts.push(AliasTemplatePart::Literal(literal.to_string()));
        remaining = rest;
    }

    Ok(AliasTemplate {
        raw: raw_template.to_string(),
        parts,
    })
}

fn capture_token_raw(input: &str) -> IResult<&str, &str> {
    delimited(char('{'), take_till(|ch| ch == '}'), char('}')).parse(input)
}

fn parse_capture_transforms(
    rule_index: usize,
    capture_token: &str,
) -> Result<Vec<AliasTransform>, AliasRuleValidationError> {
    if capture_token.trim().is_empty() {
        return Ok(vec![]);
    }

    let (_, transform_names) = capture_transform_names(capture_token).map_err(|_| {
        AliasRuleValidationError::to_error(
            rule_index,
            format!(
                "unsupported to template token `{{{capture_token}}}`; expected pipe-delimited transforms"
            ),
        )
    })?;

    transform_names
        .into_iter()
        .map(|name| parse_transform(rule_index, name))
        .collect()
}

fn capture_transform_names(input: &str) -> IResult<&str, Vec<&str>> {
    all_consuming(separated_list1(
        delimited(space0, char('|'), space0),
        transform_name,
    ))
    .parse(input)
}

fn transform_name(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((tag("_"), nom::character::complete::alpha1)),
        many0(alt((tag("_"), nom::character::complete::alphanumeric1))),
    ))
    .parse(input)
}

fn parse_transform(
    rule_index: usize,
    name: &str,
) -> Result<AliasTransform, AliasRuleValidationError> {
    all_consuming(known_transform)
        .parse(name)
        .map(|(_, transform)| transform)
        .map_err(|_| {
            AliasRuleValidationError::to_error(
                rule_index,
                format!("unsupported to template transform `{name}`"),
            )
        })
}

fn known_transform(input: &str) -> IResult<&str, AliasTransform> {
    value(AliasTransform::Snakecase, tag("snakecase")).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn raw_rule(from: &str, to: &str) -> RawAliasRule {
        RawAliasRule {
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    fn assert_error(
        errors: &[AliasRuleValidationError],
        rule_index: usize,
        field: AliasRuleField,
        message_fragment: &str,
    ) {
        let matching_error = errors
            .iter()
            .find(|error| error.rule_index == rule_index && error.field == field)
            .unwrap_or_else(|| {
                panic!("missing error for rule {rule_index}, field {field:?}: {errors:?}")
            });

        assert!(
            matching_error.message.contains(message_fragment),
            "expected `{}` to contain `{}`",
            matching_error.message,
            message_fragment,
        );
    }

    #[test]
    fn compile_alias_rules_accepts_single_wildcard_and_known_tokens() {
        let result = compile_alias_rules(&[
            raw_rule("*?", "be_{}"),
            raw_rule("has_*?", "have_{}"),
            raw_rule("*Validator", "{snakecase}"),
        ]);

        assert_eq!(
            result,
            Ok(vec![
                AliasRule {
                    from: AliasFromPattern {
                        raw: "*?".to_string(),
                        prefix: String::new(),
                        suffix: "?".to_string(),
                    },
                    to: AliasTemplate {
                        raw: "be_{}".to_string(),
                        parts: vec![
                            AliasTemplatePart::Literal("be_".to_string()),
                            AliasTemplatePart::Capture(vec![]),
                        ],
                    },
                },
                AliasRule {
                    from: AliasFromPattern {
                        raw: "has_*?".to_string(),
                        prefix: "has_".to_string(),
                        suffix: "?".to_string(),
                    },
                    to: AliasTemplate {
                        raw: "have_{}".to_string(),
                        parts: vec![
                            AliasTemplatePart::Literal("have_".to_string()),
                            AliasTemplatePart::Capture(vec![]),
                        ],
                    },
                },
                AliasRule {
                    from: AliasFromPattern {
                        raw: "*Validator".to_string(),
                        prefix: "".to_string(),
                        suffix: "Validator".to_string(),
                    },
                    to: AliasTemplate {
                        raw: "{snakecase}".to_string(),
                        parts: vec![AliasTemplatePart::Capture(vec![AliasTransform::Snakecase])],
                    },
                },
            ])
        );
    }

    #[test]
    fn compile_alias_rules_accepts_pipe_delimited_transform_syntax() {
        let result = compile_alias_rules(&[raw_rule("*Validator", "{snakecase | snakecase}")]);

        assert_eq!(
            result,
            Ok(vec![AliasRule {
                from: AliasFromPattern {
                    raw: "*Validator".to_string(),
                    prefix: String::new(),
                    suffix: "Validator".to_string(),
                },
                to: AliasTemplate {
                    raw: "{snakecase | snakecase}".to_string(),
                    parts: vec![AliasTemplatePart::Capture(vec![
                        AliasTransform::Snakecase,
                        AliasTransform::Snakecase,
                    ])],
                },
            }])
        );
    }

    #[test]
    fn compile_alias_rules_rejects_from_without_wildcard() {
        let result = compile_alias_rules(&[raw_rule("admin?", "be_{}")]).unwrap_err();

        assert_error(&result, 0, AliasRuleField::From, "exactly one `*` wildcard");
    }

    #[test]
    fn compile_alias_rules_rejects_from_with_multiple_wildcards() {
        let result = compile_alias_rules(&[raw_rule("*foo*", "be_{}")]).unwrap_err();

        assert_error(&result, 0, AliasRuleField::From, "exactly one `*` wildcard");
    }

    #[test]
    fn compile_alias_rules_rejects_unknown_to_transform() {
        let result = compile_alias_rules(&[raw_rule("*?", "be_{camelcase}")]).unwrap_err();

        assert_error(
            &result,
            0,
            AliasRuleField::To,
            "unsupported to template transform",
        );
    }

    #[test]
    fn compile_alias_rules_rejects_bad_transform_pipeline() {
        let result = compile_alias_rules(&[raw_rule("*?", "be_{snakecase | }")]).unwrap_err();

        assert_error(
            &result,
            0,
            AliasRuleField::To,
            "unsupported to template token",
        );
    }

    #[test]
    fn compile_alias_rules_rejects_unclosed_to_token() {
        let result = compile_alias_rules(&[raw_rule("*?", "be_{")]).unwrap_err();

        assert_error(&result, 0, AliasRuleField::To, "unclosed `{` token");
    }

    #[test]
    fn compile_alias_rules_reports_multiple_validation_classes_order_independently() {
        let result = compile_alias_rules(&[
            raw_rule("admin?", "be_{}"),
            raw_rule("*?", "be_{camelcase}"),
        ])
        .unwrap_err();

        assert_eq!(result.len(), 2);
        assert_error(&result, 0, AliasRuleField::From, "exactly one `*` wildcard");
        assert_error(
            &result,
            1,
            AliasRuleField::To,
            "unsupported to template transform",
        );
    }

    proptest! {
        #[test]
        fn expands_wildcard_alias_for_question_methods(stem in "[A-Za-z_][A-Za-z0-9_]{0,20}") {
            let rules = compile_alias_rules(&[raw_rule("*?", "be_{}")]).expect("compile rules");
            let input = format!("{stem}?");
            let candidates = expand_alias_candidates(&input, &rules);
            let expected_alias = format!("be_{stem}");

            prop_assert!(candidates.contains(&input));
            prop_assert!(candidates.contains(&expected_alias));
        }

        #[test]
        fn snakecase_transform_is_deterministic(class_name in "[A-Z][A-Za-z0-9]{0,20}Validator") {
            let rules = compile_alias_rules(&[raw_rule("*Validator", "{snakecase}")]).expect("compile rules");

            let first = expand_alias_candidates(&class_name, &rules);
            let second = expand_alias_candidates(&class_name, &rules);

            prop_assert_eq!(first, second);
        }
    }
}
