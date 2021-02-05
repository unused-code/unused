use ignore::{WalkBuilder, WalkState};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq)]
pub struct CodebaseFiles {
    pub paths: Vec<PathBuf>,
}

impl CodebaseFiles {
    pub fn all() -> CodebaseFiles {
        let mut builder = WalkBuilder::new("./");
        builder.hidden(false);
        builder.filter_entry(|entry| entry.path() != Path::new("./.git"));

        let results = Arc::new(Mutex::new(vec![]));

        builder.build_parallel().run(|| {
            Box::new(|result| {
                if let Ok(entry) = result {
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        let mut results = results.lock().unwrap();
                        let path = entry.path().strip_prefix("./").unwrap_or(entry.path());

                        results.push(path.to_path_buf());
                    }
                }

                WalkState::Continue
            })
        });

        let mut paths = results.lock().unwrap().to_vec();
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
