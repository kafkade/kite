pub mod battery;
pub mod cpu;
pub mod disk;
pub mod docker;
pub mod gpu;
pub mod k8s;
pub mod memory;
pub mod network;
pub mod process;
pub mod remote;
pub mod sensor;

use anyhow::Result;

/// Trait for all data collectors (CPU, memory, disk, network, etc.).
///
/// Each collector gathers one category of system metrics.
/// Collectors are polled by the application on each data tick.
pub trait Collector: Send {
    /// Collect fresh metrics. Called on each data tick.
    fn collect(&mut self) -> Result<()>;
}
