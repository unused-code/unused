use super::usage_likelihood::UsageLikelihoodStatus;
use project_configuration::{Assertion, ValueMatcher};
use std::default::Default;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use token_search::TokenSearchResult;

pub struct AnalysisFilter {
    pub usage_likelihood_filter: Vec<UsageLikelihoodStatus>,
    pub sort_order: SortOrder,
    ignored_by_path: Vec<Assertion>,
}

pub enum SortOrder {
    Ascending(OrderField),
    Descending(OrderField),
}

#[derive(Copy, Clone, Debug)]
pub enum OrderField {
    Token,
    File,
}

impl Default for OrderField {
    fn default() -> Self {
        OrderField::Token
    }
}

impl FromStr for OrderField {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "file" => Ok(OrderField::File),
            "token" => Ok(OrderField::Token),
            val => Err(String::from(format!("Unable to parse order: {}", val))),
        }
    }
}

impl OrderField {
    pub fn variants() -> Vec<&'static str> {
        vec!["token", "file"]
    }
}

impl AnalysisFilter {
    pub fn set_order_field(&mut self, field: OrderField) {
        match self.sort_order {
            SortOrder::Ascending(_) => self.sort_order = SortOrder::Ascending(field),
            SortOrder::Descending(_) => self.sort_order = SortOrder::Descending(field),
        }
    }

    pub fn set_order_ascending(&mut self) {
        match &self.sort_order {
            SortOrder::Descending(field) => self.sort_order = SortOrder::Ascending(*field),
            _ => (),
        }
    }

    pub fn set_order_descending(&mut self) {
        match &self.sort_order {
            SortOrder::Ascending(field) => self.sort_order = SortOrder::Descending(*field),
            _ => (),
        }
    }

    pub fn set_ignored(&mut self, substrings: Vec<String>) {
        self.ignored_by_path = substrings
            .into_iter()
            .map(|s| Assertion::PathAssertion(ValueMatcher::Contains(s)))
            .collect()
    }

    pub fn ignores_path(&self, result: &TokenSearchResult) -> bool {
        if self.ignored_by_path.len() > 0 {
            !self.ignored_by_path.iter().any(|a| a.matches(result))
        } else {
            true
        }
    }
}

impl Default for AnalysisFilter {
    fn default() -> Self {
        AnalysisFilter {
            usage_likelihood_filter: vec![UsageLikelihoodStatus::High],
            sort_order: SortOrder::Ascending(OrderField::Token),
            ignored_by_path: vec![],
        }
    }
}

impl Display for OrderField {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            OrderField::Token => write!(f, "token"),
            OrderField::File => write!(f, "file"),
        }
    }
}

impl Display for SortOrder {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SortOrder::Ascending(field) => write!(f, "{} (asc)", field),
            SortOrder::Descending(field) => write!(f, "{} (desc)", field),
        }
    }
}
