use project_configuration::{PathPrefix, ProjectConfiguration};
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Add;
use std::path::{Path, PathBuf};
use token_search::TokenSearchResult;

#[derive(Clone, Copy, Serialize)]
pub struct Counts {
    pub file_count: usize,
    pub occurrence_count: usize,
}

#[derive(PartialEq)]
pub enum FileType {
    ApplicationFile,
    TestFile,
    ConfigFile,
    UnknownFile,
}

impl Default for Counts {
    fn default() -> Self {
        Self {
            file_count: 0,
            occurrence_count: 0,
        }
    }
}

impl Add for Counts {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            file_count: self.file_count + other.file_count,
            occurrence_count: self.occurrence_count + other.occurrence_count,
        }
    }
}

impl Counts {
    pub fn from_occurrences<A>(occurrences: &HashMap<A, usize>) -> Self {
        Self {
            file_count: occurrences.keys().len(),
            occurrence_count: occurrences.values().sum(),
        }
    }
}

#[derive(Serialize)]
pub struct FileTypeCounts {
    pub app: Counts,
    pub config: Counts,
    pub test: Counts,
    pub unknown: Counts,
}

impl Default for FileTypeCounts {
    fn default() -> Self {
        Self {
            app: Counts::default(),
            config: Counts::default(),
            test: Counts::default(),
            unknown: Counts::default(),
        }
    }
}

impl FileTypeCounts {
    pub fn new(
        project_configuration: &ProjectConfiguration,
        token_search_result: &TokenSearchResult,
    ) -> Self {
        let results = token_search_result.occurrences.clone();
        let mut app: HashMap<&PathBuf, usize> = HashMap::new();
        let mut config: HashMap<&PathBuf, usize> = HashMap::new();
        let mut test: HashMap<&PathBuf, usize> = HashMap::new();
        let mut unknown: HashMap<&PathBuf, usize> = HashMap::new();

        for (k, v) in &results {
            if Self::is_application_file(project_configuration, k) {
                app.insert(k, *v);
            }

            if Self::is_config_file(project_configuration, k) {
                config.insert(k, *v);
            }

            if Self::is_test_file(project_configuration, k) {
                test.insert(k, *v);
            }

            if Self::is_unknown_file(project_configuration, k) {
                unknown.insert(k, *v);
            }
        }

        Self {
            app: Counts::from_occurrences(&app),
            config: Counts::from_occurrences(&config),
            test: Counts::from_occurrences(&test),
            unknown: Counts::from_occurrences(&unknown),
        }
    }

    pub fn total(&self) -> Counts {
        vec![self.app, self.config, self.test, self.unknown]
            .iter()
            .fold(Counts::default(), |mut acc, o| {
                acc = acc + *o;
                acc
            })
    }

    fn is_application_file(project_configuration: &ProjectConfiguration, path: &PathBuf) -> bool {
        Self::file_type(project_configuration, path) == FileType::ApplicationFile
    }

    fn is_config_file(project_configuration: &ProjectConfiguration, path: &PathBuf) -> bool {
        Self::file_type(project_configuration, path) == FileType::ConfigFile
    }

    fn is_test_file(project_configuration: &ProjectConfiguration, path: &PathBuf) -> bool {
        Self::file_type(project_configuration, path) == FileType::TestFile
    }

    fn is_unknown_file(project_configuration: &ProjectConfiguration, path: &PathBuf) -> bool {
        Self::file_type(project_configuration, path) == FileType::UnknownFile
    }

    fn file_type(project_configuration: &ProjectConfiguration, path: &PathBuf) -> FileType {
        if Self::compare_file(path, &project_configuration.application_file) {
            FileType::ApplicationFile
        } else if Self::compare_file(path, &project_configuration.test_file) {
            FileType::TestFile
        } else if Self::compare_file(path, &project_configuration.config_file) {
            FileType::ConfigFile
        } else {
            FileType::UnknownFile
        }
    }

    fn compare_file(file: &PathBuf, paths: &[PathPrefix]) -> bool {
        paths.iter().any(|p| p.compare(Path::new(file)))
    }
}
