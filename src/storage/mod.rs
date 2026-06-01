// Data persistence and logging
// Inspired by atop's logging capabilities

use crate::core::Result;
use std::path::PathBuf;

pub struct Logger {
    log_path: PathBuf,
    enabled: bool,
}

impl Logger {
    pub fn new(log_path: PathBuf) -> Self {
        Self {
            log_path,
            enabled: true,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn log<T>(&self, data: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        // TODO: Implement data logging
        Ok(())
    }
}
