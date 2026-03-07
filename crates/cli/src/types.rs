use clap::ValueEnum;
use read_ctags::Language;
use std::fmt::{Display, Formatter};
use token_analysis::OrderField;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LanguageExtension {
    Css,
    Ex,
    Exs,
    Elm,
    Html,
    Json,
    Js,
    Jsx,
    Md,
    Py,
    Rb,
    Rs,
    Scss,
    Sh,
    Svg,
    Ts,
    Tsx,
    Xml,
}

impl From<LanguageExtension> for Language {
    fn from(ext: LanguageExtension) -> Self {
        match ext {
            LanguageExtension::Css => Language::CSS,
            LanguageExtension::Ex | LanguageExtension::Exs => Language::Elixir,
            LanguageExtension::Elm => Language::Elm,
            LanguageExtension::Html => Language::HTML,
            LanguageExtension::Json => Language::JSON,
            LanguageExtension::Js | LanguageExtension::Jsx => Language::JavaScript,
            LanguageExtension::Md => Language::Markdown,
            LanguageExtension::Py => Language::Python,
            LanguageExtension::Rb => Language::Ruby,
            LanguageExtension::Rs => Language::Rust,
            LanguageExtension::Scss => Language::SCSS,
            LanguageExtension::Sh => Language::Sh,
            LanguageExtension::Svg => Language::SVG,
            LanguageExtension::Ts | LanguageExtension::Tsx => Language::TypeScript,
            LanguageExtension::Xml => Language::XML,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SortOrder {
    File,
    Token,
}

impl From<SortOrder> for OrderField {
    fn from(sort_order: SortOrder) -> Self {
        match sort_order {
            SortOrder::File => OrderField::File,
            SortOrder::Token => OrderField::Token,
        }
    }
}

impl From<OrderField> for SortOrder {
    fn from(order_field: OrderField) -> Self {
        match order_field {
            OrderField::File => SortOrder::File,
            OrderField::Token => SortOrder::Token,
        }
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        OrderField::default().into()
    }
}

impl Display for SortOrder {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SortOrder::File => write!(f, "file"),
            SortOrder::Token => write!(f, "token"),
        }
    }
}

#[derive(Debug, Clone, Default, ValueEnum)]
pub enum Format {
    #[default]
    Standard,
    Compact,
    Json,
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Format::Standard => write!(f, "standard"),
            Format::Compact => write!(f, "compact"),
            Format::Json => write!(f, "json"),
        }
    }
}
