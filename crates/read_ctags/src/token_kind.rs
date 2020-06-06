use super::language::Language;
use serde::{Deserialize, Serialize};

/// TokenKind is an enum which represents different types of tokens
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum TokenKind {
    Class,
    Id,
    Selector,
    // Elixir
    Macro,
    Callback,
    Delegate,
    Exception,
    Function,
    Guard,
    Implementation,
    Module,
    Operator,
    Protocol,
    Record,
    Test,
    Type,
    // Elm
    Alias,
    Constructor,
    Namespace,
    Port,
    // HTML
    Stylesheet,
    Script,
    Anchor,
    Heading1,
    Heading2,
    Heading3,
    // JSON
    Array,
    Boolean,
    Number,
    Object,
    String,
    Null,
    // JavaScript
    Constant,
    Getter,
    Setter,
    Generator,
    Method,
    Property,
    Variable,
    // Markdown
    Subsection,
    L4Subsection,
    Chapter,
    Section,
    SubSubsection,
    L5Subsection,
    // Python
    Member,
    // Ruby
    SingletonMethod,
    RSpecDescribe,
    // Rust
    Field,
    Struct,
    Typedef,
    // SCSS
    Placeholder,
    Mixin,
    Parameter,
    // Sh
    Heredoc,
    // SVG
    Def,
    // TypeScript
    Enumerator,
    Enum,
    Interface,
    Local,
    // XML
    NSPrefix,
    Root,
    Undefined,
    MissingLanguageToken(Language, char),
    Unknown(char),
}

static LOOKUP: [(Language, char, TokenKind); 107] = [
    (Language::CSS, 'c', TokenKind::Class),
    (Language::CSS, 'i', TokenKind::Id),
    (Language::CSS, 's', TokenKind::Selector),
    (Language::Elixir, 'a', TokenKind::Macro),
    (Language::Elixir, 'c', TokenKind::Callback),
    (Language::Elixir, 'd', TokenKind::Delegate),
    (Language::Elixir, 'e', TokenKind::Exception),
    (Language::Elixir, 'f', TokenKind::Function),
    (Language::Elixir, 'g', TokenKind::Guard),
    (Language::Elixir, 'i', TokenKind::Implementation),
    (Language::Elixir, 'm', TokenKind::Module),
    (Language::Elixir, 'o', TokenKind::Operator),
    (Language::Elixir, 'p', TokenKind::Protocol),
    (Language::Elixir, 'r', TokenKind::Record),
    (Language::Elixir, 't', TokenKind::Test),
    (Language::Elixir, 'y', TokenKind::Type),
    (Language::Elm, 'a', TokenKind::Alias),
    (Language::Elm, 'c', TokenKind::Constructor),
    (Language::Elm, 'f', TokenKind::Function),
    (Language::Elm, 'm', TokenKind::Module),
    (Language::Elm, 'n', TokenKind::Namespace),
    (Language::Elm, 'p', TokenKind::Port),
    (Language::Elm, 't', TokenKind::Type),
    (Language::HTML, 'C', TokenKind::Stylesheet),
    (Language::HTML, 'I', TokenKind::Id),
    (Language::HTML, 'J', TokenKind::Script),
    (Language::HTML, 'a', TokenKind::Anchor),
    (Language::HTML, 'c', TokenKind::Class),
    (Language::HTML, 'h', TokenKind::Heading1),
    (Language::HTML, 'i', TokenKind::Heading2),
    (Language::HTML, 'j', TokenKind::Heading3),
    (Language::JSON, 'a', TokenKind::Array),
    (Language::JSON, 'b', TokenKind::Boolean),
    (Language::JSON, 'n', TokenKind::Number),
    (Language::JSON, 'o', TokenKind::Object),
    (Language::JSON, 's', TokenKind::String),
    (Language::JSON, 'z', TokenKind::Null),
    (Language::JavaScript, 'C', TokenKind::Constant),
    (Language::JavaScript, 'G', TokenKind::Getter),
    (Language::JavaScript, 'S', TokenKind::Setter),
    (Language::JavaScript, 'c', TokenKind::Class),
    (Language::JavaScript, 'f', TokenKind::Function),
    (Language::JavaScript, 'g', TokenKind::Generator),
    (Language::JavaScript, 'm', TokenKind::Method),
    (Language::JavaScript, 'p', TokenKind::Property),
    (Language::JavaScript, 'v', TokenKind::Variable),
    (Language::Markdown, 'S', TokenKind::Subsection),
    (Language::Markdown, 'T', TokenKind::L4Subsection),
    (Language::Markdown, 'c', TokenKind::Chapter),
    (Language::Markdown, 's', TokenKind::Section),
    (Language::Markdown, 't', TokenKind::SubSubsection),
    (Language::Markdown, 'u', TokenKind::L5Subsection),
    (Language::Python, 'I', TokenKind::Namespace),
    (Language::Python, 'c', TokenKind::Class),
    (Language::Python, 'f', TokenKind::Function),
    (Language::Python, 'i', TokenKind::Module),
    (Language::Python, 'l', TokenKind::Local),
    (Language::Python, 'm', TokenKind::Member),
    (Language::Python, 'v', TokenKind::Variable),
    (Language::Python, 'x', TokenKind::Unknown('x')),
    (Language::Python, 'z', TokenKind::Parameter),
    (Language::Ruby, 'S', TokenKind::SingletonMethod),
    (Language::Ruby, 'c', TokenKind::Class),
    (Language::Ruby, 'f', TokenKind::Method),
    (Language::Ruby, 'm', TokenKind::Module),
    (Language::Ruby, 'd', TokenKind::RSpecDescribe),
    (Language::Rust, 'M', TokenKind::Macro),
    (Language::Rust, 'P', TokenKind::Method),
    (Language::Rust, 'c', TokenKind::Implementation),
    (Language::Rust, 'e', TokenKind::Enumerator),
    (Language::Rust, 'f', TokenKind::Function),
    (Language::Rust, 'g', TokenKind::Enum),
    (Language::Rust, 'i', TokenKind::Interface),
    (Language::Rust, 'm', TokenKind::Field),
    (Language::Rust, 'n', TokenKind::Module),
    (Language::Rust, 's', TokenKind::Struct),
    (Language::Rust, 't', TokenKind::Typedef),
    (Language::Rust, 'v', TokenKind::Variable),
    (Language::SCSS, 'P', TokenKind::Placeholder),
    (Language::SCSS, 'c', TokenKind::Class),
    (Language::SCSS, 'f', TokenKind::Function),
    (Language::SCSS, 'i', TokenKind::Id),
    (Language::SCSS, 'm', TokenKind::Mixin),
    (Language::SCSS, 'v', TokenKind::Variable),
    (Language::SCSS, 'z', TokenKind::Parameter),
    (Language::Sh, 'a', TokenKind::Alias),
    (Language::Sh, 'f', TokenKind::Function),
    (Language::Sh, 'h', TokenKind::Heredoc),
    (Language::Sh, 's', TokenKind::Script),
    (Language::TypeScript, 'C', TokenKind::Constant),
    (Language::TypeScript, 'G', TokenKind::Generator),
    (Language::TypeScript, 'a', TokenKind::Alias),
    (Language::TypeScript, 'c', TokenKind::Class),
    (Language::TypeScript, 'e', TokenKind::Enumerator),
    (Language::TypeScript, 'f', TokenKind::Function),
    (Language::TypeScript, 'g', TokenKind::Enum),
    (Language::TypeScript, 'i', TokenKind::Interface),
    (Language::TypeScript, 'l', TokenKind::Local),
    (Language::TypeScript, 'm', TokenKind::Method),
    (Language::TypeScript, 'n', TokenKind::Namespace),
    (Language::TypeScript, 'p', TokenKind::Property),
    (Language::TypeScript, 'v', TokenKind::Variable),
    (Language::TypeScript, 'z', TokenKind::Parameter),
    (Language::XML, 'i', TokenKind::Id),
    (Language::XML, 'n', TokenKind::NSPrefix),
    (Language::XML, 'r', TokenKind::Root),
    (Language::SVG, 'd', TokenKind::Def),
];

impl TokenKind {
    /// Construct a TokenKind given a language (or lack thereof) with a character
    ///
    /// This is based off of Universal Ctags' generated list:
    ///   $ ctags --list-kinds-full
    ///
    /// This list is not comprehensive in that it is based off of the languages accounted for.
    pub fn from_ctag(lang: Option<Language>, identifier: char) -> TokenKind {
        LOOKUP
            .iter()
            .filter(|(l, c, _)| Some(*l) == lang && *c == identifier)
            .nth(0)
            .map(|(_, _, t)| t.clone())
            .unwrap_or(match lang {
                Some(Language::SVG) => Self::from_ctag(Some(Language::XML), identifier),
                Some(l) => TokenKind::MissingLanguageToken(l, identifier),
                None => TokenKind::Unknown(identifier),
            })
    }

    /// Calculate the character given an optional language and kind
    pub fn to_token_char(&self, lang: Option<Language>) -> char {
        match *self {
            TokenKind::Unknown(c) => c,
            TokenKind::MissingLanguageToken(_, c) => c,
            _ => LOOKUP
                .iter()
                .filter(|(l, _, t)| Some(*l) == lang && t == self)
                .nth(0)
                .map(|(_, c, _)| c.clone())
                .unwrap_or(match lang {
                    Some(Language::SVG) => self.to_token_char(Some(Language::XML)),
                    _ => ' ',
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bidirectional_support() {
        for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars() {
            for l in Language::all() {
                assert_eq!(c, TokenKind::from_ctag(Some(l), c).to_token_char(Some(l)));
            }
            assert_eq!(c, TokenKind::from_ctag(None, c).to_token_char(None));
        }
    }
}
