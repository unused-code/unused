use super::{
    super::project_configurations_loader::load_and_parse_config,
    check_up::{CheckUp, Status},
};
use project_configuration::ProjectConfigurations;

pub struct LoadedProjectConfigurations(Result<ProjectConfigurations, String>);

impl LoadedProjectConfigurations {
    pub fn new() -> Self {
        LoadedProjectConfigurations(load_and_parse_config())
    }

    fn config_keys(&self) -> Vec<String> {
        match &self.0 {
            Ok(configurations) => configurations.project_config_names(),
            Err(_) => vec![],
        }
    }
}

impl CheckUp for LoadedProjectConfigurations {
    fn name(&self) -> &'static str {
        "Does the loaded configuration have available project types?"
    }

    fn status(&self) -> Status {
        match &self.0 {
            Ok(_) => {
                if self.config_keys().is_empty() {
                    Status::Warn(
                        "No project configurations were loaded; using default config instead."
                            .to_string(),
                    )
                } else {
                    Status::OK(format!(
                        "Loaded the following project configurations: {}",
                        self.config_keys().join(", ")
                    ))
                }
            }
            Err(error) => Status::Error(error.clone()),
        }
    }
}
