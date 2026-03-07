#![warn(clippy::pedantic)]

mod alias_rules;
mod loader;
mod project_configuration;
mod value_assertion;

pub use crate::project_configuration::{PathPrefix, ProjectConfiguration};
pub use alias_rules::{
    AliasFromPattern, AliasRule, AliasRuleField, AliasRuleValidationError, AliasTemplate,
    AliasTemplatePart, RawAliasRule, compile_alias_rules,
};
pub use loader::ProjectConfigurations;
pub use value_assertion::{Assertion, AssertionConflict, ValueMatcher};
