use serde::Serialize;
use std::path::Path;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub enum Language {
    CSS,
    Elixir,
    Elm,
    HTML,
    JSON,
    JavaScript,
    Markdown,
    Ruby,
    Rust,
    SCSS,
    Sh,
    SVG,
    TypeScript,
    XML,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Language::CSS => write!(f, "CSS"),
            Language::Elixir => write!(f, "Elixir"),
            Language::Elm => write!(f, "Elm"),
            Language::HTML => write!(f, "HTML"),
            Language::JSON => write!(f, "JSON"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::Markdown => write!(f, "Markdown"),
            Language::Ruby => write!(f, "Ruby"),
            Language::Rust => write!(f, "Rust"),
            Language::SCSS => write!(f, "SCSS"),
            Language::Sh => write!(f, "Shell"),
            Language::SVG => write!(f, "SVG"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::XML => write!(f, "XML"),
        }
    }
}

impl Language {
    pub fn from_path(path: &str) -> Option<Language> {
        match Path::new(path).extension() {
            Some(v) => v.to_str().and_then(|x| Language::from_str(x).ok()),
            None => Some(Language::Sh),
        }
    }

    pub fn extensions() -> Vec<&'static str> {
        vec![
            "css", "ex", "exs", "elm", "html", "json", "js", "jsx", "md", "rb", "rs", "scss", "sh",
            "svg", "ts", "tsx", "xml",
        ]
    }
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "css" => Ok(Language::CSS),
            "ex" => Ok(Language::Elixir),
            "exs" => Ok(Language::Elixir),
            "elm" => Ok(Language::Elm),
            "html" => Ok(Language::HTML),
            "json" => Ok(Language::JSON),
            "js" => Ok(Language::JavaScript),
            "jsx" => Ok(Language::JavaScript),
            "md" => Ok(Language::Markdown),
            "rb" => Ok(Language::Ruby),
            "rs" => Ok(Language::Rust),
            "scss" => Ok(Language::SCSS),
            "sh" => Ok(Language::Sh),
            "svg" => Ok(Language::SVG),
            "ts" => Ok(Language::TypeScript),
            "tsx" => Ok(Language::TypeScript),
            "xml" => Ok(Language::XML),
            "" => Ok(Language::Sh),
            ext => Err(String::from(format!(
                "Unable to parse file extension: {}",
                ext
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use totems::assert_ok;

    #[test]
    fn calculates_common_files() {
        assert_eq!(Language::from_path("../foo/bar.rb"), Some(Language::Ruby));
        assert_eq!(Language::from_path("/tmp/foo.md"), Some(Language::Markdown));
        assert_eq!(Language::from_path("bin/rails"), Some(Language::Sh));
        assert_eq!(Language::from_path("file.unknown"), None);
    }

    #[test]
    fn all_extensions_are_supported() {
        for ext in Language::extensions().iter() {
            assert_ok!(Language::from_str(ext));
        }
    }
}
