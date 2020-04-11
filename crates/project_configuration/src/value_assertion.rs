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
                .defined_paths()
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
}

impl ValueMatcher {
    pub fn check(&self, haystack: &str) -> bool {
        match self {
            ValueMatcher::StartsWith(v) => haystack.starts_with(v),
            ValueMatcher::EndsWith(v) => haystack.ends_with(v),
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
}
