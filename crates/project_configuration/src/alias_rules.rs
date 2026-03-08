use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RawAliasRule {
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AliasRule {
    pub from: AliasFromPattern,
    pub to: AliasTemplate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AliasFromPattern {
    pub raw: String,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AliasTemplate {
    pub raw: String,
    pub parts: Vec<AliasTemplatePart>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AliasTemplatePart {
    Literal(String),
    Capture,
    SnakecaseCapture,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AliasRuleField {
    From,
    To,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AliasRuleValidationError {
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

fn parse_from_pattern(
    rule_index: usize,
    raw_pattern: &str,
) -> Result<AliasFromPattern, AliasRuleValidationError> {
    let wildcard_indices = raw_pattern
        .char_indices()
        .filter_map(|(idx, ch)| (ch == '*').then_some(idx))
        .collect::<Vec<usize>>();

    if wildcard_indices.len() != 1 {
        return Err(AliasRuleValidationError::from_error(
            rule_index,
            "from pattern must contain exactly one `*` wildcard".to_string(),
        ));
    }

    let wildcard_index = wildcard_indices[0];
    let prefix = raw_pattern[..wildcard_index].to_string();
    let suffix = raw_pattern[(wildcard_index + 1)..].to_string();

    Ok(AliasFromPattern {
        raw: raw_pattern.to_string(),
        prefix,
        suffix,
    })
}

fn parse_template(
    rule_index: usize,
    raw_template: &str,
) -> Result<AliasTemplate, AliasRuleValidationError> {
    let mut parts = Vec::new();
    let mut cursor = 0;

    while cursor < raw_template.len() {
        let open_offset = raw_template[cursor..].find('{');

        match open_offset {
            Some(offset) => {
                let open = cursor + offset;
                if open > cursor {
                    parts.push(AliasTemplatePart::Literal(
                        raw_template[cursor..open].to_string(),
                    ));
                }

                let token_start = open + 1;
                let close_offset = raw_template[token_start..].find('}');

                let close = match close_offset {
                    Some(offset) => token_start + offset,
                    None => {
                        return Err(AliasRuleValidationError::to_error(
                            rule_index,
                            "to template has an unclosed `{` token".to_string(),
                        ));
                    }
                };

                let token = &raw_template[token_start..close];
                match token {
                    "" => parts.push(AliasTemplatePart::Capture),
                    "snakecase" => parts.push(AliasTemplatePart::SnakecaseCapture),
                    _ => {
                        return Err(AliasRuleValidationError::to_error(
                            rule_index,
                            format!("unsupported to template token `{{{token}}}`"),
                        ));
                    }
                }

                cursor = close + 1;
            }
            None => {
                parts.push(AliasTemplatePart::Literal(
                    raw_template[cursor..].to_string(),
                ));
                cursor = raw_template.len();
            }
        }
    }

    Ok(AliasTemplate {
        raw: raw_template.to_string(),
        parts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
                        prefix: "".to_string(),
                        suffix: "?".to_string(),
                    },
                    to: AliasTemplate {
                        raw: "be_{}".to_string(),
                        parts: vec![
                            AliasTemplatePart::Literal("be_".to_string()),
                            AliasTemplatePart::Capture,
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
                            AliasTemplatePart::Capture,
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
                        parts: vec![AliasTemplatePart::SnakecaseCapture],
                    },
                },
            ])
        );
    }

    #[test]
    fn compile_alias_rules_rejects_from_without_wildcard() {
        let result = compile_alias_rules(&[raw_rule("admin?", "be_{}")]).unwrap_err();

        assert_error(
            &result,
            0,
            AliasRuleField::From,
            "exactly one `*` wildcard",
        );
    }

    #[test]
    fn compile_alias_rules_rejects_from_with_multiple_wildcards() {
        let result = compile_alias_rules(&[raw_rule("*foo*", "be_{}")]).unwrap_err();

        assert_error(
            &result,
            0,
            AliasRuleField::From,
            "exactly one `*` wildcard",
        );
    }

    #[test]
    fn compile_alias_rules_rejects_unknown_to_token() {
        let result = compile_alias_rules(&[raw_rule("*?", "be_{camelcase}")]).unwrap_err();

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
        assert_error(&result, 1, AliasRuleField::To, "unsupported to template token");
    }
}
