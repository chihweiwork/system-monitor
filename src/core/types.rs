use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct UpdateInterval(Duration);

impl UpdateInterval {
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }

    pub fn as_duration(&self) -> Duration {
        self.0
    }
}

impl Default for UpdateInterval {
    fn default() -> Self {
        Self(Duration::from_secs(1))
    }
}
