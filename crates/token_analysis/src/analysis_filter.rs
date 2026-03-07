use super::usage_likelihood::UsageLikelihoodStatus;
use project_configuration::{Assertion, ValueMatcher};
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

#[derive(Copy, Clone, Debug, Default)]
pub enum OrderField {
    #[default]
    Token,
    File,
}

impl FromStr for OrderField {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "file" => Ok(OrderField::File),
            "token" => Ok(OrderField::Token),
            val => Err(format!("Unable to parse order: {val}")),
        }
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
        if let SortOrder::Descending(field) = &self.sort_order {
            self.sort_order = SortOrder::Ascending(*field);
        }
    }

    pub fn set_order_descending(&mut self) {
        if let SortOrder::Ascending(field) = &self.sort_order {
            self.sort_order = SortOrder::Descending(*field);
        }
    }

    pub fn set_ignored(&mut self, substrings: Vec<String>) {
        self.ignored_by_path = substrings
            .into_iter()
            .map(|s| Assertion::PathAssertion(ValueMatcher::Contains(s)))
            .collect();
    }

    #[must_use]
    pub fn ignores_path(&self, result: &TokenSearchResult) -> bool {
        if self.ignored_by_path.is_empty() {
            true
        } else {
            !self.ignored_by_path.iter().any(|a| a.matches(result))
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
            SortOrder::Ascending(field) => write!(f, "{field} (asc)"),
            SortOrder::Descending(field) => write!(f, "{field} (desc)"),
        }
    }
}
