use anyhow::Result;
use sysinfo::System;

use crate::collector::Collector;
use crate::util::ring_buffer::RingBuffer;

/// Collects CPU metrics: usage, frequencies, model info, load averages, and uptime.
pub struct CpuCollector {
    system: System,
    total_usage: f64,
    per_core_usage: Vec<f64>,
    frequencies: Vec<u64>,
    cpu_model: String,
    core_count: usize,
    thread_count: usize,
    load_averages: (f64, f64, f64),
    uptime_secs: u64,
    usage_history: RingBuffer<f64>,
}

impl CpuCollector {
    pub fn new(history_capacity: usize) -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();

        let thread_count = system.cpus().len();
        let core_count = System::physical_core_count().unwrap_or(thread_count);
        let cpu_model = system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_default();

        Self {
            system,
            total_usage: 0.0,
            per_core_usage: Vec::new(),
            frequencies: Vec::new(),
            cpu_model,
            core_count,
            thread_count,
            load_averages: (0.0, 0.0, 0.0),
            uptime_secs: 0,
            usage_history: RingBuffer::new(history_capacity),
        }
    }

    pub fn total_usage(&self) -> f64 {
        self.total_usage
    }

    pub fn per_core_usage(&self) -> &[f64] {
        &self.per_core_usage
    }

    pub fn cpu_model(&self) -> &str {
        &self.cpu_model
    }

    pub fn core_count(&self) -> usize {
        self.core_count
    }

    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    pub fn frequencies(&self) -> &[u64] {
        &self.frequencies
    }

    pub fn load_averages(&self) -> (f64, f64, f64) {
        self.load_averages
    }

    pub fn uptime_secs(&self) -> u64 {
        self.uptime_secs
    }

    pub fn usage_history(&self) -> &RingBuffer<f64> {
        &self.usage_history
    }

    /// Set CPU usage directly (used by replay mode).
    pub fn set_usage(&mut self, total: f64, per_core: Vec<f64>) {
        self.total_usage = total;
        self.per_core_usage = per_core;
        self.usage_history.push(total);
    }
}

impl Collector for CpuCollector {
    fn collect(&mut self) -> Result<()> {
        self.system.refresh_cpu_all();

        self.total_usage = self.system.global_cpu_usage() as f64;
        self.usage_history.push(self.total_usage);

        self.per_core_usage = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .collect();

        self.frequencies = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.frequency())
            .collect();

        #[cfg(not(target_os = "windows"))]
        {
            let load = System::load_average();
            self.load_averages = (load.one, load.five, load.fifteen);
        }
        #[cfg(target_os = "windows")]
        {
            self.load_averages = (0.0, 0.0, 0.0);
        }

        self.uptime_secs = System::uptime();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_properly() {
        let collector = CpuCollector::new(300);
        assert_eq!(collector.usage_history().capacity(), 300);
        assert!(collector.usage_history().is_empty());
        assert!(collector.core_count() > 0);
        assert!(collector.thread_count() > 0);
        assert!(!collector.cpu_model().is_empty());
    }

    #[test]
    fn collect_succeeds() {
        let mut collector = CpuCollector::new(300);
        assert!(collector.collect().is_ok());
    }

    #[test]
    fn values_in_expected_ranges() {
        let mut collector = CpuCollector::new(300);
        // First collect may return 0; second collect gives real values.
        collector.collect().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(250));
        collector.collect().unwrap();

        assert!(
            collector.total_usage() >= 0.0 && collector.total_usage() <= 100.0,
            "total_usage out of range: {}",
            collector.total_usage()
        );

        for (i, &usage) in collector.per_core_usage().iter().enumerate() {
            assert!(
                (0.0..=100.0).contains(&usage),
                "core {i} usage out of range: {usage}"
            );
        }

        assert_eq!(collector.per_core_usage().len(), collector.thread_count());
        assert_eq!(collector.frequencies().len(), collector.thread_count());
        assert!(collector.uptime_secs() > 0);
        assert_eq!(collector.usage_history().len(), 2);
    }
}
