use anyhow::Result;
use sysinfo::Components;

use crate::collector::Collector;
use crate::util::ring_buffer::RingBuffer;

/// A single sensor reading snapshot.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SensorReading {
    pub label: String,
    pub current_temp: f32,
    pub max_temp: f32,
    pub critical_temp: Option<f32>,
}

/// Collects temperature and hardware sensor data via `sysinfo::Components`.
pub struct SensorCollector {
    components: Components,
    readings: Vec<SensorReading>,
    cpu_temp_history: RingBuffer<f64>,
    /// Stable label used to identify the CPU temperature sensor across refreshes.
    cpu_sensor_label: Option<String>,
}

impl SensorCollector {
    pub fn new(history_capacity: usize) -> Self {
        let components = Components::new_with_refreshed_list();

        // Scan for a CPU-related sensor to track over time.
        // Prefer labels containing "Package", then "CPU", then "Core".
        let cpu_sensor_label = Self::detect_cpu_sensor(&components);

        Self {
            components,
            readings: Vec::new(),
            cpu_temp_history: RingBuffer::new(history_capacity),
            cpu_sensor_label,
        }
    }

    /// Heuristic search for the best CPU temperature sensor label.
    fn detect_cpu_sensor(components: &Components) -> Option<String> {
        let candidates: Vec<&str> = components.iter().map(|c| c.label()).collect();

        // Priority order: "Package" > "CPU" > "Core"
        for keyword in &["package", "cpu", "core"] {
            if let Some(label) = candidates
                .iter()
                .find(|l| l.to_lowercase().contains(keyword))
            {
                return Some(label.to_string());
            }
        }

        // Fall back to the first sensor if any exist.
        candidates.first().map(|l| l.to_string())
    }

    pub fn readings(&self) -> &[SensorReading] {
        &self.readings
    }

    pub fn cpu_temp_history(&self) -> &RingBuffer<f64> {
        &self.cpu_temp_history
    }

    /// Returns the current CPU temperature if a CPU sensor was detected.
    pub fn cpu_temp(&self) -> Option<f32> {
        let label = self.cpu_sensor_label.as_deref()?;
        self.readings
            .iter()
            .find(|r| r.label == label)
            .map(|r| r.current_temp)
    }

    /// Returns `true` if any sensor readings are available.
    pub fn has_sensors(&self) -> bool {
        !self.readings.is_empty()
    }
}

impl Collector for SensorCollector {
    fn collect(&mut self) -> Result<()> {
        // Refresh existing component values without rebuilding the list.
        self.components.refresh(false);

        self.readings = self
            .components
            .iter()
            .filter_map(|c| {
                Some(SensorReading {
                    label: c.label().to_string(),
                    current_temp: c.temperature()?,
                    max_temp: c.max().unwrap_or(0.0),
                    critical_temp: c.critical(),
                })
            })
            .collect();

        // Track CPU temperature history.
        if let Some(ref label) = self.cpu_sensor_label {
            if let Some(reading) = self.readings.iter().find(|r| r.label == *label) {
                self.cpu_temp_history.push(reading.current_temp as f64);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes() {
        let collector = SensorCollector::new(300);
        assert_eq!(collector.cpu_temp_history().capacity(), 300);
        assert!(collector.cpu_temp_history().is_empty());
    }

    #[test]
    fn collect_succeeds() {
        let mut collector = SensorCollector::new(300);
        assert!(collector.collect().is_ok());
    }

    #[test]
    fn has_sensors_works() {
        let mut collector = SensorCollector::new(300);
        collector.collect().unwrap();
        // Result depends on CI environment — just ensure no panic.
        let _ = collector.has_sensors();
    }

    #[test]
    fn readings_have_valid_temps() {
        let mut collector = SensorCollector::new(300);
        collector.collect().unwrap();

        for reading in collector.readings() {
            assert!(
                reading.current_temp >= 0.0 && reading.current_temp <= 200.0,
                "temperature out of range for {}: {}",
                reading.label,
                reading.current_temp
            );
        }
    }

    #[test]
    fn cpu_temp_history_tracks() {
        let mut collector = SensorCollector::new(300);
        collector.collect().unwrap();
        collector.collect().unwrap();
        collector.collect().unwrap();

        if collector.cpu_sensor_label.is_some() && collector.has_sensors() {
            assert!(
                !collector.cpu_temp_history().is_empty(),
                "CPU temp history should grow when a CPU sensor is detected"
            );
        }
    }
}
