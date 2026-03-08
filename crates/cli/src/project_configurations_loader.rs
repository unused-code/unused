use project_configuration::ProjectConfigurations;
use std::fs;
use std::io;
use std::path::Path;

pub fn load_and_parse_config() -> Result<ProjectConfigurations, String> {
    match file_path_in_home_dir(".config/unused/unused.yml") {
        Some(path) => load_from_user_config_path(&path),
        None => ProjectConfigurations::parse(ProjectConfigurations::default_yaml().as_str())
            .map_err(|error| format!("Failed to parse bundled default config: {error}")),
    }
}

fn load_from_user_config_path(path: &str) -> Result<ProjectConfigurations, String> {
    if !Path::new(path).exists() {
        return ProjectConfigurations::parse(ProjectConfigurations::default_yaml().as_str())
            .map_err(|error| format!("Failed to parse bundled default config: {error}"));
    }

    let contents = read_file(path)
        .map_err(|error| format!("Failed to read project configuration from `{path}`: {error}"))?;

    ProjectConfigurations::parse(&contents)
        .map_err(|error| format!("Failed to parse project configuration from `{path}`: {error}"))
}

fn file_path_in_home_dir(file_name: &str) -> Option<String> {
    dirs_next::home_dir().and_then(|ref p| {
        Path::new(p)
            .join(file_name)
            .to_str()
            .map(std::borrow::ToOwned::to_owned)
    })
}

fn read_file(filename: &str) -> Result<String, io::Error> {
    let contents = fs::read_to_string(filename)?;

    Ok(contents)
}
