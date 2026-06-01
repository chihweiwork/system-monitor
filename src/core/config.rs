use super::UpdateInterval;

#[derive(Debug, Clone)]
pub struct Config {
    pub update_interval: UpdateInterval,
    pub enable_logging: bool,
    pub log_path: Option<std::path::PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_interval: UpdateInterval::default(),
            enable_logging: false,
            log_path: None,
        }
    }
}
