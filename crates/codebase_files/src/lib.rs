use std::process::{Command, Output};

pub struct CodebaseFiles {
    pub paths: Vec<String>,
}

impl CodebaseFiles {
    pub fn all() -> CodebaseFiles {
        let output = Command::new("git").arg("ls-files").output();
        let mut paths = Self::process_ls_files(output);
        paths.extend(Self::process_ls_files(
            Command::new("git")
                .arg("ls-files")
                .arg("--others")
                .arg("--exclude-standard")
                .output(),
        ));

        paths.sort();
        paths.dedup();
        CodebaseFiles { paths }
    }

    fn process_ls_files<T>(output: Result<Output, T>) -> Vec<String> {
        match output {
            Ok(o) => {
                if o.status.success() {
                    std::str::from_utf8(&o.stdout).map_or(vec![], |v| {
                        v.lines().map(|k| k.to_string()).collect::<Vec<String>>()
                    })
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
}
