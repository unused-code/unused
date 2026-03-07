#![warn(clippy::pedantic)]

use ignore::{WalkBuilder, WalkState};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq)]
pub struct CodebaseFiles {
    pub paths: Vec<PathBuf>,
}

impl CodebaseFiles {
    #[must_use]
    pub fn all() -> CodebaseFiles {
        let mut builder = WalkBuilder::new("./");
        builder.hidden(false);
        builder.filter_entry(|entry| entry.path() != Path::new("./.git"));

        let results = Arc::new(Mutex::new(vec![]));

        builder.build_parallel().run(|| {
            Box::new(|result| {
                if let Ok(entry) = result
                    && entry.file_type().is_some_and(|ft| ft.is_file())
                {
                    let mut results = match results.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    let path = entry.path().strip_prefix("./").unwrap_or(entry.path());

                    results.push(path.to_path_buf());
                }

                WalkState::Continue
            })
        });

        let mut paths = match results.lock() {
            Ok(guard) => guard.to_vec(),
            Err(poisoned) => poisoned.into_inner().to_vec(),
        };
        paths.sort();

        CodebaseFiles { paths }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_prefix() {
        assert_eq!(
            CodebaseFiles::all(),
            CodebaseFiles {
                paths: vec![PathBuf::from("Cargo.toml"), PathBuf::from("src/lib.rs")]
            }
        );
    }
}
