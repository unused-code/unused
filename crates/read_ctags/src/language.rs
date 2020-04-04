use serde::Serialize;
use std::path::Path;

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

impl Language {
    pub fn from_path(path: &str) -> Option<Language> {
        match Path::new(path).extension().and_then(|v| v.to_str()) {
            Some("css") => Some(Language::CSS),
            Some("ex") => Some(Language::Elixir),
            Some("exs") => Some(Language::Elixir),
            Some("elm") => Some(Language::Elm),
            Some("html") => Some(Language::HTML),
            Some("json") => Some(Language::JSON),
            Some("js") => Some(Language::JavaScript),
            Some("jsx") => Some(Language::JavaScript),
            Some("md") => Some(Language::Markdown),
            Some("rb") => Some(Language::Ruby),
            Some("rs") => Some(Language::Rust),
            Some("scss") => Some(Language::SCSS),
            Some("svg") => Some(Language::SVG),
            Some("ts") => Some(Language::TypeScript),
            Some("tsx") => Some(Language::TypeScript),
            Some("xml") => Some(Language::XML),
            None => Some(Language::Sh),
            _ => None,
        }
    }
}

#[test]
fn calculates_common_files() {
    assert_eq!(Language::from_path("../foo/bar.rb"), Some(Language::Ruby));
    assert_eq!(Language::from_path("/tmp/foo.md"), Some(Language::Markdown));
    assert_eq!(Language::from_path("bin/rails"), Some(Language::Sh));
    assert_eq!(Language::from_path("file.unknown"), None);
}
