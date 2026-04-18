use anyhow::Result;

use crate::collector::Collector;

/// Aggregated battery state for display.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BatterySnapshot {
    pub percentage: f32,
    pub state: BatteryState,
    pub time_to_full: Option<std::time::Duration>,
    pub time_to_empty: Option<std::time::Duration>,
    pub power_draw_watts: Option<f64>,
    /// Battery health: energy_full / energy_full_design * 100.
    pub health_percent: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    Empty,
    Unknown,
}

/// Collects battery metrics via `starship-battery` (when the `battery` feature is enabled).
pub struct BatteryCollector {
    #[cfg(feature = "battery")]
    manager: Option<starship_battery::Manager>,
    snapshot: Option<BatterySnapshot>,
}

// ── Feature-gated implementation: battery ON ────────────────────────────────

#[cfg(feature = "battery")]
impl BatteryCollector {
    pub fn new() -> Self {
        let manager = starship_battery::Manager::new().ok();
        Self {
            manager,
            snapshot: None,
        }
    }
}

#[cfg(feature = "battery")]
impl Collector for BatteryCollector {
    fn collect(&mut self) -> Result<()> {
        use starship_battery::units::{energy::watt_hour, power::watt, time::second};

        let manager = match &self.manager {
            Some(m) => m,
            None => {
                self.snapshot = None;
                return Ok(());
            }
        };

        let batteries = match manager.batteries() {
            Ok(b) => b,
            Err(_) => {
                self.snapshot = None;
                return Ok(());
            }
        };

        let mut sum_energy: f64 = 0.0;
        let mut sum_energy_full: f64 = 0.0;
        let mut sum_energy_full_design: f64 = 0.0;
        let mut sum_energy_rate: f64 = 0.0;
        let mut first_state: Option<BatteryState> = None;
        let mut any_charging = false;
        let mut any_discharging = false;
        let mut time_to_full: Option<std::time::Duration> = None;
        let mut time_to_empty: Option<std::time::Duration> = None;
        let mut count = 0u32;

        for battery_result in batteries {
            let battery = match battery_result {
                Ok(b) => b,
                Err(_) => continue,
            };

            count += 1;

            let energy = battery.energy().get::<watt_hour>() as f64;
            let energy_full = battery.energy_full().get::<watt_hour>() as f64;
            let energy_full_design = battery.energy_full_design().get::<watt_hour>() as f64;
            let energy_rate = battery.energy_rate().get::<watt>() as f64;

            sum_energy += energy;
            sum_energy_full += energy_full;
            sum_energy_full_design += energy_full_design;
            sum_energy_rate += energy_rate;

            let state = map_state(battery.state());
            if first_state.is_none() {
                first_state = Some(state);
            }
            match state {
                BatteryState::Charging => any_charging = true,
                BatteryState::Discharging => any_discharging = true,
                _ => {}
            }

            if time_to_full.is_none() {
                time_to_full = battery.time_to_full().map(|t| {
                    let secs = t.get::<second>();
                    std::time::Duration::from_secs_f32(secs)
                });
            }
            if time_to_empty.is_none() {
                time_to_empty = battery.time_to_empty().map(|t| {
                    let secs = t.get::<second>();
                    std::time::Duration::from_secs_f32(secs)
                });
            }
        }

        if count == 0 {
            self.snapshot = None;
            return Ok(());
        }

        let percentage = if sum_energy_full > 0.0 {
            ((sum_energy / sum_energy_full) * 100.0).clamp(0.0, 100.0) as f32
        } else {
            0.0
        };

        let health_percent = if sum_energy_full_design > 0.0 {
            Some((sum_energy_full / sum_energy_full_design) * 100.0)
        } else {
            None
        };

        let aggregate_state = if any_charging {
            BatteryState::Charging
        } else if any_discharging {
            BatteryState::Discharging
        } else {
            first_state.unwrap_or(BatteryState::Unknown)
        };

        let power_draw = if sum_energy_rate > 0.0 {
            Some(sum_energy_rate)
        } else {
            None
        };

        self.snapshot = Some(BatterySnapshot {
            percentage,
            state: aggregate_state,
            time_to_full,
            time_to_empty,
            power_draw_watts: power_draw,
            health_percent,
        });

        Ok(())
    }
}

#[cfg(feature = "battery")]
fn map_state(state: starship_battery::State) -> BatteryState {
    match state {
        starship_battery::State::Charging => BatteryState::Charging,
        starship_battery::State::Discharging => BatteryState::Discharging,
        starship_battery::State::Full => BatteryState::Full,
        starship_battery::State::Empty => BatteryState::Empty,
        _ => BatteryState::Unknown,
    }
}

// ── Feature-gated implementation: battery OFF ───────────────────────────────

#[cfg(not(feature = "battery"))]
impl BatteryCollector {
    pub fn new() -> Self {
        Self { snapshot: None }
    }
}

#[cfg(not(feature = "battery"))]
impl Collector for BatteryCollector {
    fn collect(&mut self) -> Result<()> {
        Ok(())
    }
}

// ── Shared getters (always available) ───────────────────────────────────────

impl BatteryCollector {
    #[allow(dead_code)]
    pub fn has_battery(&self) -> bool {
        self.snapshot.is_some()
    }

    #[allow(dead_code)]
    pub fn snapshot(&self) -> Option<&BatterySnapshot> {
        self.snapshot.as_ref()
    }

    /// Compact status string for a status bar.
    ///
    /// Returns an empty string when no battery is present (desktops).
    pub fn format_compact(&self) -> String {
        match &self.snapshot {
            None => String::new(),
            Some(snap) => match snap.state {
                BatteryState::Full => format!("BAT: {:.0}% FULL", snap.percentage),
                BatteryState::Charging => format!("BAT: {:.0}% CHG", snap.percentage),
                BatteryState::Discharging => {
                    if let Some(watts) = snap.power_draw_watts {
                        format!("BAT: {:.0}% ({:.1}W)", snap.percentage, watts)
                    } else {
                        format!("BAT: {:.0}%", snap.percentage)
                    }
                }
                _ => format!("BAT: {:.0}%", snap.percentage),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes() {
        let _collector = BatteryCollector::new();
    }

    #[test]
    fn collect_succeeds() {
        let mut collector = BatteryCollector::new();
        assert!(collector.collect().is_ok());
    }

    #[test]
    fn has_battery_consistent() {
        let mut collector = BatteryCollector::new();
        collector.collect().ok();
        assert_eq!(collector.has_battery(), collector.snapshot().is_some());
    }

    #[test]
    fn format_compact_discharging_with_power() {
        let mut collector = BatteryCollector::new();
        collector.snapshot = Some(BatterySnapshot {
            percentage: 85.0,
            state: BatteryState::Discharging,
            time_to_full: None,
            time_to_empty: Some(std::time::Duration::from_secs(3600)),
            power_draw_watts: Some(12.3),
            health_percent: Some(95.0),
        });
        assert_eq!(collector.format_compact(), "BAT: 85% (12.3W)");
    }

    #[test]
    fn format_compact_charging() {
        let mut collector = BatteryCollector::new();
        collector.snapshot = Some(BatterySnapshot {
            percentage: 42.0,
            state: BatteryState::Charging,
            time_to_full: Some(std::time::Duration::from_secs(1800)),
            time_to_empty: None,
            power_draw_watts: Some(45.0),
            health_percent: Some(98.0),
        });
        assert_eq!(collector.format_compact(), "BAT: 42% CHG");
    }

    #[test]
    fn format_compact_full() {
        let mut collector = BatteryCollector::new();
        collector.snapshot = Some(BatterySnapshot {
            percentage: 100.0,
            state: BatteryState::Full,
            time_to_full: None,
            time_to_empty: None,
            power_draw_watts: None,
            health_percent: Some(90.0),
        });
        assert_eq!(collector.format_compact(), "BAT: 100% FULL");
    }

    #[test]
    fn format_compact_no_battery() {
        let collector = BatteryCollector::new();
        assert_eq!(collector.format_compact(), "");
    }

    #[test]
    fn stub_works() {
        let collector = BatteryCollector::new();
        assert!(!collector.has_battery());
        assert!(collector.snapshot().is_none());
        assert_eq!(collector.format_compact(), "");
    }
}
