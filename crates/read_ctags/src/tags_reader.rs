use super::{CtagItem, CtagsParseError, TagsFile};
use std::convert::From;
use std::default::Default;
use std::env::current_dir;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Error;
use std::path::PathBuf;
use std::process::Command;

/// TagsReader provides a mechanism for attempting to read multiple ctags files until the first is
/// found
pub struct TagsReader {
    filenames: Vec<PathBuf>,
}

/// A struct capturing possible failures when attempting to find and read tags files
#[derive(Debug)]
pub enum ReadCtagsError {
    /// No tags file found
    ///
    /// This provides the paths attempted
    NoCtagsFile(Vec<PathBuf>, io::Error),
    /// Error parsing tags
    CtagsParseError(CtagsParseError),
}

impl From<CtagsParseError> for ReadCtagsError {
    fn from(err: CtagsParseError) -> Self {
        ReadCtagsError::CtagsParseError(err)
    }
}

impl Display for ReadCtagsError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            ReadCtagsError::NoCtagsFile(ref file_list, ref err) => write!(
                f,
                "Unable to find ctags file (searched in {}): {}",
                file_list
                    .iter()
                    .filter_map(|f| f.to_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                err
            ),
            ReadCtagsError::CtagsParseError(ref err) => write!(f, "{}", err),
        }
    }
}

fn git_path() -> Option<PathBuf> {
    match Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .output()
    {
        Ok(o) => {
            if o.status.success() {
                std::str::from_utf8(&o.stdout)
                    .ok()
                    .map(|k| PathBuf::from(k.trim_end()))
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

fn cwd_tags_paths(cwd: PathBuf) -> Vec<PathBuf> {
    vec![cwd.join("tags"), cwd.join("tmp/tags")]
}

impl Default for TagsReader {
    fn default() -> Self {
        let mut filenames = vec![];

        if let Ok(current_dir) = current_dir() {
            if let Some(app_git_path) = git_path() {
                if app_git_path == PathBuf::from(".git") {
                    filenames.push(current_dir.join(app_git_path).join("tags"));
                    filenames.extend(cwd_tags_paths(current_dir));
                } else {
                    filenames.extend(cwd_tags_paths(current_dir));
                    filenames.push(app_git_path.join("tags"));
                    filenames.push(app_git_path.join("../tags"));
                    filenames.push(app_git_path.join("../tmp/tags"));
                }
            } else {
                filenames.push(current_dir.join("tags"));
                filenames.push(current_dir.join("tmp/tags"));
            }
        }

        TagsReader { filenames }
    }
}

impl TagsReader {
    /// Loads and parses the first tags file it finds
    pub fn load(&self) -> Result<TagsFile, ReadCtagsError> {
        self.read().and_then(|(ctags_path, contents)| {
            CtagItem::parse(ctags_path, &contents).map_err(|e| e.into())
        })
    }

    /// Override the default set of paths with a user-provided one
    pub fn for_tags_file(&mut self, path: PathBuf) -> &mut Self {
        self.filenames = vec![path];
        self
    }

    fn read(&self) -> Result<(PathBuf, String), ReadCtagsError> {
        Self::first_success(
            &self.filenames,
            Error::new(io::ErrorKind::Other, "No file provided"),
            read_to_string_lossy,
        )
        .map_err(|e| ReadCtagsError::NoCtagsFile(self.filenames.clone(), e))
    }

    fn first_success<A, B, C, F>(values: &[A], default: C, f: F) -> Result<(A, B), C>
    where
        A: Clone,
        F: Fn(A) -> Result<B, C>,
    {
        let mut outcome = Err(default);
        for x in values.iter() {
            outcome = f(x.clone()).map(|v| (x.clone(), v));
            if outcome.is_ok() {
                break;
            }
        }
        outcome
    }
}

fn read_to_string_lossy<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}
