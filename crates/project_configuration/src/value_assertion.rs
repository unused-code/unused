use std::collections::HashSet;
use token_search::TokenSearchResult;

#[derive(Clone, Debug, PartialEq)]
pub enum Assertion {
    PathAssertion(ValueMatcher),
    TokenAssertion(ValueMatcher),
}

impl Assertion {
    pub fn matches(&self, token_search_result: &TokenSearchResult) -> bool {
        match self {
            Assertion::PathAssertion(matcher) => token_search_result
                .token
                .defined_paths
                .iter()
                .any(|path| matcher.check(path)),
            Assertion::TokenAssertion(matcher) => matcher.check(&token_search_result.token.token),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueMatcher {
    StartsWith(String),
    EndsWith(String),
    Equals(String),
    ExactMatchOnAnyOf(HashSet<String>),
    StartsWithCapital,
}

impl ValueMatcher {
    pub fn check(&self, haystack: &str) -> bool {
        match self {
            ValueMatcher::StartsWith(v) => haystack.starts_with(v),
            ValueMatcher::EndsWith(v) => haystack.ends_with(v),
            ValueMatcher::Equals(v) => haystack == v,
            ValueMatcher::ExactMatchOnAnyOf(vs) => vs.contains(haystack),
            ValueMatcher::StartsWithCapital => haystack.starts_with(|v: char| v.is_uppercase()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foo() -> String {
        String::from("foo")
    }

    fn bar() -> String {
        String::from("bar")
    }

    #[test]
    fn matches_starts_with() {
        assert!(ValueMatcher::StartsWith(foo()).check(&"foobar"));
        assert!(!ValueMatcher::StartsWith(bar()).check(&"foobar"));
    }

    #[test]
    fn matches_ends_with() {
        assert!(ValueMatcher::EndsWith(bar()).check(&"foobar"));
        assert!(!ValueMatcher::EndsWith(foo()).check(&"foobar"));
    }

    #[test]
    fn matches_any_of() {
        let values: HashSet<_> = vec![foo(), bar()].iter().cloned().collect();

        assert!(ValueMatcher::ExactMatchOnAnyOf(values.clone()).check(&"foo"));
        assert!(ValueMatcher::ExactMatchOnAnyOf(values.clone()).check(&"bar"));
        assert!(!ValueMatcher::ExactMatchOnAnyOf(values.clone()).check(&"foobar"));
    }

    #[test]
    fn matches_capital() {
        assert!(ValueMatcher::StartsWithCapital.check(&"Foo"));
        assert!(!ValueMatcher::StartsWithCapital.check(&"foo"));
    }

    #[test]
    fn matches_equals() {
        assert!(ValueMatcher::Equals(foo()).check("foo"));
        assert!(!ValueMatcher::Equals(foo()).check("Foo"));
        assert!(!ValueMatcher::Equals(foo()).check(" foo"));
        assert!(!ValueMatcher::Equals(foo()).check("foo "));
    }
}
