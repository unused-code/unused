use super::{
    super::project_configurations_loader::load_and_parse_config,
    check_up::{CheckUp, Status},
};
use project_configuration::ProjectConfigurations;

pub struct LoadedProjectConfigurations(ProjectConfigurations);

impl LoadedProjectConfigurations {
    pub fn new() -> Self {
        LoadedProjectConfigurations(load_and_parse_config())
    }

    fn config_keys(&self) -> Vec<String> {
        self.0.project_config_names()
    }
}

impl CheckUp for LoadedProjectConfigurations {
    fn name(&self) -> &str {
        "Does the loaded configuration have available project types?"
    }

    fn status(&self) -> Status {
        if self.config_keys().is_empty() {
            Status::Warn(
                "No project configurations were loaded; using default config instead.".to_string(),
            )
        } else {
            Status::OK(format!(
                "Loaded the following project configurations: {}",
                self.config_keys().join(", ")
            ))
        }
    }
}
