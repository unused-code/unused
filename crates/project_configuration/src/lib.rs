#![warn(clippy::pedantic)]

mod alias_rules;
mod loader;
mod project_configuration;
mod value_assertion;

pub use crate::project_configuration::{PathPrefix, ProjectConfiguration};
pub use alias_rules::{
    AliasFromPattern, AliasRule, AliasRuleField, AliasRuleValidationError, AliasTemplate,
    AliasTemplatePart, AliasTransform, RawAliasRule, apply_alias_transforms, compile_alias_rules,
    expand_alias_candidates, render_alias_template,
};
pub use loader::ProjectConfigurations;
pub use value_assertion::{Assertion, AssertionConflict, ValueMatcher};
