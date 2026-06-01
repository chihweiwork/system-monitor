// Data collection modules for various system metrics
// Each submodule handles a specific monitoring domain

pub mod cpu;
pub mod memory;
pub mod disk;
pub mod network;
pub mod io;
pub mod process;

use crate::core::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Collector {
    type Output;

    async fn collect(&mut self) -> Result<Self::Output>;
}
