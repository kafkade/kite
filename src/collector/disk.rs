use std::time::Instant;

use anyhow::Result;
use sysinfo::Disks;

use crate::collector::Collector;
use crate::util::ring_buffer::RingBuffer;

/// Information about a single mounted filesystem / partition.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub usage_percent: f64,
}

/// Collects disk space and I/O metrics for all mounted filesystems.
pub struct DiskCollector {
    disks: Disks,
    disk_info: Vec<DiskInfo>,
    total_read_bytes_sec: f64,
    total_write_bytes_sec: f64,
    read_history: RingBuffer<f64>,
    write_history: RingBuffer<f64>,
    last_collect: Instant,
}

#[allow(dead_code)]
impl DiskCollector {
    pub fn new(history_capacity: usize) -> Self {
        let disks = Disks::new_with_refreshed_list();
        let disk_info = Self::build_disk_info(&disks);

        Self {
            disks,
            disk_info,
            total_read_bytes_sec: 0.0,
            total_write_bytes_sec: 0.0,
            read_history: RingBuffer::new(history_capacity),
            write_history: RingBuffer::new(history_capacity),
            last_collect: Instant::now(),
        }
    }

    fn build_disk_info(disks: &Disks) -> Vec<DiskInfo> {
        disks
            .list()
            .iter()
            .map(|d| {
                let total = d.total_space();
                let free = d.available_space();
                let used = total.saturating_sub(free);
                let usage_percent = if total > 0 {
                    (used as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                DiskInfo {
                    name: d.name().to_string_lossy().to_string(),
                    mount_point: d.mount_point().to_string_lossy().to_string(),
                    fs_type: d.file_system().to_string_lossy().to_string(),
                    total_bytes: total,
                    used_bytes: used,
                    free_bytes: free,
                    usage_percent,
                }
            })
            .collect()
    }

    pub fn disks(&self) -> &[DiskInfo] {
        &self.disk_info
    }

    pub fn total_read_bytes_sec(&self) -> f64 {
        self.total_read_bytes_sec
    }

    pub fn total_write_bytes_sec(&self) -> f64 {
        self.total_write_bytes_sec
    }

    pub fn read_history(&self) -> &RingBuffer<f64> {
        &self.read_history
    }

    pub fn write_history(&self) -> &RingBuffer<f64> {
        &self.write_history
    }

    /// Set disk data directly (used by replay mode).
    pub fn set_disk_data(
        &mut self,
        read_bytes_sec: f64,
        write_bytes_sec: f64,
        filesystems: Vec<DiskInfo>,
    ) {
        self.total_read_bytes_sec = read_bytes_sec;
        self.total_write_bytes_sec = write_bytes_sec;
        self.disk_info = filesystems;
        self.read_history.push(read_bytes_sec);
        self.write_history.push(write_bytes_sec);
    }
}

impl Collector for DiskCollector {
    fn collect(&mut self) -> Result<()> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_collect).as_secs_f64();

        self.disks.refresh(true);
        self.disk_info = Self::build_disk_info(&self.disks);

        // Sum I/O bytes since last refresh across all disks.
        let mut total_read: u64 = 0;
        let mut total_written: u64 = 0;
        for d in self.disks.list() {
            let usage = d.usage();
            total_read += usage.read_bytes;
            total_written += usage.written_bytes;
        }

        if elapsed > 0.0 {
            self.total_read_bytes_sec = total_read as f64 / elapsed;
            self.total_write_bytes_sec = total_written as f64 / elapsed;
        }

        self.read_history.push(self.total_read_bytes_sec);
        self.write_history.push(self.total_write_bytes_sec);
        self.last_collect = now;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_finds_at_least_one_disk() {
        let collector = DiskCollector::new(60);
        assert!(
            !collector.disks().is_empty(),
            "should find at least one disk"
        );
    }

    #[test]
    fn collect_succeeds() {
        let mut collector = DiskCollector::new(60);
        collector.collect().expect("collect should succeed");
    }

    #[test]
    fn usage_percent_in_range() {
        let collector = DiskCollector::new(60);
        for d in collector.disks() {
            assert!(
                (0.0..=100.0).contains(&d.usage_percent),
                "usage_percent {} out of range for {}",
                d.usage_percent,
                d.mount_point,
            );
        }
    }

    #[test]
    fn total_bytes_positive() {
        let collector = DiskCollector::new(60);
        for d in collector.disks() {
            assert!(
                d.total_bytes > 0,
                "total_bytes should be > 0 for {}",
                d.mount_point,
            );
        }
    }
}
