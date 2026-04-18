use anyhow::Result;

use crate::collector::Collector;
use crate::util::ring_buffer::RingBuffer;

/// Snapshot of a single GPU device.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GpuDevice {
    pub name: String,
    pub index: u32,
    pub utilization_gpu: Option<u32>,
    pub utilization_memory: Option<u32>,
    pub vram_used: Option<u64>,
    pub vram_total: Option<u64>,
    pub temperature: Option<u32>,
    pub fan_speed: Option<u32>,
    pub clock_graphics: Option<u32>,
    pub clock_memory: Option<u32>,
    pub power_draw: Option<f64>,
    pub power_limit: Option<f64>,
}

/// Collects GPU metrics via NVML (when the `gpu` feature is enabled).
pub struct GpuCollector {
    #[cfg(feature = "gpu")]
    nvml: Option<nvml_wrapper::Nvml>,
    devices: Vec<GpuDevice>,
    gpu_usage_history: RingBuffer<f64>,
}

// ── Feature-gated implementation: gpu ON ────────────────────────────────────

#[cfg(feature = "gpu")]
impl GpuCollector {
    pub fn new(history_capacity: usize) -> Self {
        let nvml = nvml_wrapper::Nvml::init().ok();
        Self {
            nvml,
            devices: Vec::new(),
            gpu_usage_history: RingBuffer::new(history_capacity),
        }
    }
}

#[cfg(feature = "gpu")]
impl Collector for GpuCollector {
    fn collect(&mut self) -> Result<()> {
        use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};

        let nvml = match &self.nvml {
            Some(n) => n,
            None => {
                self.devices.clear();
                return Ok(());
            }
        };

        let count = match nvml.device_count() {
            Ok(c) => c,
            Err(_) => {
                self.devices.clear();
                return Ok(());
            }
        };

        let mut new_devices = Vec::with_capacity(count as usize);

        for idx in 0..count {
            let device = match nvml.device_by_index(idx) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let utilization = device.utilization_rates().ok();

            new_devices.push(GpuDevice {
                name: device.name().ok().unwrap_or_default(),
                index: idx,
                utilization_gpu: utilization.as_ref().map(|u| u.gpu),
                utilization_memory: utilization.map(|u| u.memory),
                vram_used: device.memory_info().ok().map(|m| m.used),
                vram_total: device.memory_info().ok().map(|m| m.total),
                temperature: device.temperature(TemperatureSensor::Gpu).ok(),
                fan_speed: device.fan_speed(0).ok(),
                clock_graphics: device.clock_info(Clock::Graphics).ok(),
                clock_memory: device.clock_info(Clock::Memory).ok(),
                power_draw: device.power_usage().ok().map(|p| p as f64 / 1000.0),
                power_limit: device
                    .enforced_power_limit()
                    .ok()
                    .map(|p| p as f64 / 1000.0),
            });
        }

        // Track first GPU utilization in history
        if let Some(first) = new_devices.first() {
            let usage = first.utilization_gpu.unwrap_or(0) as f64;
            self.gpu_usage_history.push(usage);
        }

        self.devices = new_devices;
        Ok(())
    }
}

// ── Feature-gated implementation: gpu OFF ───────────────────────────────────

#[cfg(not(feature = "gpu"))]
impl GpuCollector {
    pub fn new(history_capacity: usize) -> Self {
        Self {
            devices: Vec::new(),
            gpu_usage_history: RingBuffer::new(history_capacity),
        }
    }
}

#[cfg(not(feature = "gpu"))]
impl Collector for GpuCollector {
    fn collect(&mut self) -> Result<()> {
        Ok(())
    }
}

// ── Shared getters (always available) ───────────────────────────────────────

impl GpuCollector {
    pub fn devices(&self) -> &[GpuDevice] {
        &self.devices
    }

    pub fn gpu_usage_history(&self) -> &RingBuffer<f64> {
        &self.gpu_usage_history
    }

    pub fn has_gpu(&self) -> bool {
        !self.devices.is_empty()
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes() {
        let collector = GpuCollector::new(300);
        assert_eq!(collector.gpu_usage_history().capacity(), 300);
        assert!(collector.gpu_usage_history().is_empty());
    }

    #[test]
    fn collect_succeeds() {
        let mut collector = GpuCollector::new(300);
        assert!(collector.collect().is_ok());
    }

    #[test]
    fn has_gpu_consistent() {
        let mut collector = GpuCollector::new(300);
        collector.collect().ok();
        assert_eq!(collector.has_gpu(), !collector.devices().is_empty());
    }

    #[test]
    fn stub_works() {
        // Verify the no-GPU path: empty devices, no history, etc.
        let collector = GpuCollector::new(60);
        assert!(!collector.has_gpu());
        assert_eq!(collector.device_count(), 0);
        assert!(collector.devices().is_empty());
        assert!(collector.gpu_usage_history().is_empty());
    }
}
