use dirs_next;
use project_configuration::ProjectConfigurations;
use std::fs;
use std::io;
use std::path::Path;

pub fn load_and_parse_config() -> ProjectConfigurations {
    let contents = file_path_in_home_dir(".config/unused/unused.yml")
        .and_then(|path| read_file(&path).ok())
        .unwrap_or(ProjectConfigurations::default_yaml());
    ProjectConfigurations::parse(&contents)
}

fn file_path_in_home_dir(file_name: &str) -> Option<String> {
    dirs_next::home_dir()
        .and_then(|ref p| Path::new(p).join(file_name).to_str().map(|v| v.to_owned()))
}

fn read_file(filename: &str) -> Result<String, io::Error> {
    let contents = fs::read_to_string(filename)?;

    Ok(contents)
}
