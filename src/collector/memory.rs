use anyhow::Result;
use sysinfo::System;

use crate::collector::Collector;
use crate::util::ring_buffer::RingBuffer;

pub struct MemoryCollector {
    system: System,
    total_ram: u64,
    used_ram: u64,
    free_ram: u64,
    available_ram: u64,
    swap_total: u64,
    swap_used: u64,
    swap_free: u64,
    ram_history: RingBuffer<f64>,
    swap_history: RingBuffer<f64>,
}

impl MemoryCollector {
    pub fn new(history_capacity: usize) -> Self {
        Self {
            system: System::new(),
            total_ram: 0,
            used_ram: 0,
            free_ram: 0,
            available_ram: 0,
            swap_total: 0,
            swap_used: 0,
            swap_free: 0,
            ram_history: RingBuffer::new(history_capacity),
            swap_history: RingBuffer::new(history_capacity),
        }
    }

    pub fn total_ram(&self) -> u64 {
        self.total_ram
    }

    pub fn used_ram(&self) -> u64 {
        self.used_ram
    }

    pub fn free_ram(&self) -> u64 {
        self.free_ram
    }

    pub fn available_ram(&self) -> u64 {
        self.available_ram
    }

    pub fn ram_usage_percent(&self) -> f64 {
        if self.total_ram == 0 {
            return 0.0;
        }
        (self.used_ram as f64 / self.total_ram as f64) * 100.0
    }

    pub fn swap_total(&self) -> u64 {
        self.swap_total
    }

    pub fn swap_used(&self) -> u64 {
        self.swap_used
    }

    pub fn swap_free(&self) -> u64 {
        self.swap_free
    }

    pub fn swap_usage_percent(&self) -> f64 {
        if self.swap_total == 0 {
            return 0.0;
        }
        (self.swap_used as f64 / self.swap_total as f64) * 100.0
    }

    pub fn ram_history(&self) -> &RingBuffer<f64> {
        &self.ram_history
    }

    pub fn swap_history(&self) -> &RingBuffer<f64> {
        &self.swap_history
    }
}

impl Collector for MemoryCollector {
    fn collect(&mut self) -> Result<()> {
        self.system.refresh_memory();

        self.total_ram = self.system.total_memory();
        self.used_ram = self.system.used_memory();
        self.free_ram = self.system.free_memory();
        self.available_ram = self.system.available_memory();

        self.swap_total = self.system.total_swap();
        self.swap_used = self.system.used_swap();
        self.swap_free = self.swap_total.saturating_sub(self.swap_used);

        self.ram_history.push(self.ram_usage_percent());
        self.swap_history.push(self.swap_usage_percent());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_and_collects() {
        let mut collector = MemoryCollector::new(300);
        collector.collect().expect("collect should succeed");
        assert!(collector.total_ram() > 0, "total RAM should be > 0");
    }

    #[test]
    fn collect_succeeds() {
        let mut collector = MemoryCollector::new(300);
        assert!(collector.collect().is_ok());
        assert!(collector.collect().is_ok());
    }

    #[test]
    fn usage_percentages_in_range() {
        let mut collector = MemoryCollector::new(300);
        collector.collect().unwrap();

        let ram_pct = collector.ram_usage_percent();
        assert!(
            (0.0..=100.0).contains(&ram_pct),
            "RAM usage {ram_pct}% out of range"
        );

        let swap_pct = collector.swap_usage_percent();
        assert!(
            (0.0..=100.0).contains(&swap_pct),
            "Swap usage {swap_pct}% out of range"
        );
    }

    #[test]
    fn swap_free_equals_total_minus_used() {
        let mut collector = MemoryCollector::new(300);
        collector.collect().unwrap();

        assert_eq!(
            collector.swap_free(),
            collector.swap_total().saturating_sub(collector.swap_used())
        );
    }

    #[test]
    fn history_records_entries() {
        let mut collector = MemoryCollector::new(300);
        collector.collect().unwrap();
        collector.collect().unwrap();

        assert_eq!(collector.ram_history().len(), 2);
        assert_eq!(collector.swap_history().len(), 2);
    }
}
