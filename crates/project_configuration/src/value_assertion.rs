use super::alias_rules::{AliasRule, AliasTemplatePart};
use std::collections::HashSet;
use token_search::TokenSearchResult;

#[derive(Clone, Debug, PartialEq)]
pub enum Assertion {
    PathAssertion(ValueMatcher),
    TokenAssertion(ValueMatcher),
}

impl Assertion {
    #[must_use]
    pub fn matches(&self, token_search_result: &TokenSearchResult) -> bool {
        match self {
            Assertion::PathAssertion(matcher) => token_search_result
                .token
                .defined_paths
                .iter()
                .filter_map(|path| path.to_str())
                .any(|path| matcher.check(path)),
            Assertion::TokenAssertion(matcher) => matcher.check(&token_search_result.token.token),
        }
    }

    #[must_use]
    pub fn matcher(&self) -> &ValueMatcher {
        match self {
            Assertion::PathAssertion(matcher) | Assertion::TokenAssertion(matcher) => matcher,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AssertionConflict {
    PathConflict(Vec<Assertion>),
    TokenConflict(Vec<Assertion>),
}

impl AssertionConflict {
    #[must_use]
    pub fn assertions(&self) -> &Vec<Assertion> {
        match self {
            AssertionConflict::PathConflict(assertions)
            | AssertionConflict::TokenConflict(assertions) => assertions,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueMatcher {
    StartsWith(String),
    EndsWith(String),
    Equals(String),
    Contains(String),
    ExactMatchOnAnyOf(HashSet<String>),
    StartsWithCapital,
}

impl ValueMatcher {
    #[must_use]
    pub fn check(&self, haystack: &str) -> bool {
        match self {
            ValueMatcher::StartsWith(v) => haystack.starts_with(v),
            ValueMatcher::EndsWith(v) => haystack.ends_with(v),
            ValueMatcher::Equals(v) => haystack == v,
            ValueMatcher::Contains(v) => haystack.contains(v),
            ValueMatcher::ExactMatchOnAnyOf(vs) => vs.contains(haystack),
            ValueMatcher::StartsWithCapital => haystack.starts_with(|v: char| v.is_uppercase()),
        }
    }

    #[must_use]
    pub fn full_equals(&self) -> bool {
        matches!(
            self,
            ValueMatcher::Equals(_) | ValueMatcher::ExactMatchOnAnyOf(_)
        )
    }
}

fn expand_alias_candidates(input: &str, alias_rules: &[AliasRule]) -> HashSet<String> {
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

fn render_alias_template(capture: &str, parts: &[AliasTemplatePart]) -> String {
    parts
        .iter()
        .map(|part| match part {
            AliasTemplatePart::Literal(value) => value.to_string(),
            AliasTemplatePart::Capture => capture.to_string(),
            AliasTemplatePart::SnakecaseCapture => snakecase(capture),
        })
        .collect::<Vec<_>>()
        .join("")
}

fn snakecase(value: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = value.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && ch.is_uppercase() {
            let prev = chars[i - 1];
            let next_is_lowercase = chars.get(i + 1).is_some_and(|next| next.is_lowercase());
            if prev.is_lowercase() || (prev.is_uppercase() && next_is_lowercase) {
                out.push('_');
            }
        }

        out.extend(ch.to_lowercase());
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias_rules::{AliasFromPattern, AliasTemplate};

    fn foo() -> String {
        String::from("foo")
    }

    fn bar() -> String {
        String::from("bar")
    }

    #[test]
    fn matches_starts_with() {
        assert!(ValueMatcher::StartsWith(foo()).check("foobar"));
        assert!(!ValueMatcher::StartsWith(bar()).check("foobar"));
    }

    #[test]
    fn matches_ends_with() {
        assert!(ValueMatcher::EndsWith(bar()).check("foobar"));
        assert!(!ValueMatcher::EndsWith(foo()).check("foobar"));
    }

    #[test]
    fn matches_contains() {
        assert!(ValueMatcher::Contains(bar()).check("barar"));
        assert!(ValueMatcher::Contains(bar()).check("bar"));
        assert!(ValueMatcher::Contains(bar()).check(" bar"));
        assert!(!ValueMatcher::Contains(bar()).check(" "));
        assert!(!ValueMatcher::Contains(bar()).check("nope"));
        assert!(!ValueMatcher::Contains(bar()).check("ar"));
    }

    #[test]
    fn matches_any_of() {
        let values: HashSet<_> = [foo(), bar()].iter().cloned().collect();

        assert!(ValueMatcher::ExactMatchOnAnyOf(values.clone()).check("foo"));
        assert!(ValueMatcher::ExactMatchOnAnyOf(values.clone()).check("bar"));
        assert!(!ValueMatcher::ExactMatchOnAnyOf(values.clone()).check("foobar"));
    }

    #[test]
    fn matches_capital() {
        assert!(ValueMatcher::StartsWithCapital.check("Foo"));
        assert!(!ValueMatcher::StartsWithCapital.check("foo"));
    }

    #[test]
    fn matches_equals() {
        assert!(ValueMatcher::Equals(foo()).check("foo"));
        assert!(!ValueMatcher::Equals(foo()).check("Foo"));
        assert!(!ValueMatcher::Equals(foo()).check(" foo"));
        assert!(!ValueMatcher::Equals(foo()).check("foo "));
    }

    fn alias_rule(
        from_prefix: &str,
        from_suffix: &str,
        parts: Vec<AliasTemplatePart>,
    ) -> AliasRule {
        AliasRule {
            from: AliasFromPattern {
                raw: format!("{from_prefix}*{from_suffix}"),
                prefix: from_prefix.to_string(),
                suffix: from_suffix.to_string(),
            },
            to: AliasTemplate {
                raw: "test".to_string(),
                parts,
            },
        }
    }

    #[test]
    fn expands_alias_candidates_and_dedupes_collisions() {
        let input = "admin?";
        let candidates = expand_alias_candidates(
            input,
            &[
                alias_rule(
                    "",
                    "?",
                    vec![
                        AliasTemplatePart::Literal("be_".to_string()),
                        AliasTemplatePart::Capture,
                    ],
                ),
                alias_rule(
                    "",
                    "?",
                    vec![
                        AliasTemplatePart::Literal("be_".to_string()),
                        AliasTemplatePart::Capture,
                    ],
                ),
            ],
        );

        let expected = HashSet::from([input.to_string(), "be_admin".to_string()]);
        assert_eq!(candidates, expected);
    }

    #[test]
    fn expands_alias_candidates_with_distinct_outputs_and_drops_self_map() {
        let input = "admin?";
        let candidates = expand_alias_candidates(
            input,
            &[
                alias_rule("", "?", vec![AliasTemplatePart::Capture]),
                alias_rule(
                    "",
                    "?",
                    vec![
                        AliasTemplatePart::Literal("be_".to_string()),
                        AliasTemplatePart::Capture,
                    ],
                ),
                alias_rule(
                    "",
                    "?",
                    vec![
                        AliasTemplatePart::Literal("is_".to_string()),
                        AliasTemplatePart::Capture,
                    ],
                ),
            ],
        );

        let expected = HashSet::from([
            input.to_string(),
            "be_admin".to_string(),
            "is_admin".to_string(),
        ]);
        assert_eq!(candidates, expected);
    }

    #[test]
    fn render_alias_template_preserves_capture_characters() {
        let rendered = render_alias_template(
            "ready!",
            &[
                AliasTemplatePart::Literal("be_".to_string()),
                AliasTemplatePart::Capture,
            ],
        );

        assert_eq!(rendered, "be_ready!");
    }

    #[test]
    fn render_alias_template_supports_snakecase_capture() {
        let rendered =
            render_alias_template("HTTPValidator", &[AliasTemplatePart::SnakecaseCapture]);

        assert_eq!(rendered, "http_validator");
    }
}
