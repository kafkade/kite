pub mod rules;

use std::collections::HashMap;

use chrono::{DateTime, Local};

pub use rules::{AlertRule, Condition, Metric, Severity};

/// A fired alert with context.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FiredAlert {
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
    pub fired_at: DateTime<Local>,
    pub current_value: f64,
    pub threshold: f64,
}

/// Tracks alert state and history.
pub struct AlertEngine {
    rules: Vec<AlertRule>,
    /// How many consecutive ticks each rule has been in violation.
    violation_counts: HashMap<String, u32>,
    /// Currently active (fired) alerts.
    active_alerts: Vec<FiredAlert>,
    /// History of all alerts (most recent first), capped at max_history.
    history: Vec<FiredAlert>,
    max_history: usize,
    /// Whether terminal bell was triggered this tick (to avoid spamming).
    bell_pending: bool,
}

impl AlertEngine {
    pub fn new(rules: Vec<AlertRule>, max_history: usize) -> Self {
        Self {
            rules,
            violation_counts: HashMap::new(),
            active_alerts: Vec::new(),
            history: Vec::new(),
            max_history,
            bell_pending: false,
        }
    }

    /// Evaluate all rules against current metric values.
    pub fn evaluate(&mut self, metrics: &HashMap<Metric, f64>) {
        self.active_alerts.clear();
        self.bell_pending = false;

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let current_value = match metrics.get(&rule.metric) {
                Some(v) => *v,
                None => continue,
            };

            let in_violation = match rule.condition {
                Condition::Above => current_value > rule.threshold,
                Condition::Below => current_value < rule.threshold,
            };

            let count = self.violation_counts.entry(rule.name.clone()).or_insert(0);

            if in_violation {
                *count += 1;
                if *count >= rule.duration_ticks {
                    let alert = FiredAlert {
                        rule_name: rule.name.clone(),
                        severity: rule.severity,
                        message: format!(
                            "{}: {:.1} {} {:.1}",
                            rule.name,
                            current_value,
                            match rule.condition {
                                Condition::Above => ">",
                                Condition::Below => "<",
                            },
                            rule.threshold
                        ),
                        fired_at: Local::now(),
                        current_value,
                        threshold: rule.threshold,
                    };
                    self.active_alerts.push(alert.clone());

                    // Avoid duplicate consecutive history entries for the same rule
                    let dominated = self
                        .history
                        .first()
                        .is_some_and(|h| h.rule_name == alert.rule_name);
                    if !dominated {
                        self.history.insert(0, alert);
                        if self.history.len() > self.max_history {
                            self.history.pop();
                        }
                    }

                    if rule.severity == Severity::Critical {
                        self.bell_pending = true;
                    }
                }
            } else {
                *count = 0;
            }
        }
    }

    #[allow(dead_code)]
    pub fn active_alerts(&self) -> &[FiredAlert] {
        &self.active_alerts
    }

    #[allow(dead_code)]
    pub fn history(&self) -> &[FiredAlert] {
        &self.history
    }

    #[allow(dead_code)]
    pub fn has_active_alerts(&self) -> bool {
        !self.active_alerts.is_empty()
    }

    pub fn highest_severity(&self) -> Option<Severity> {
        self.active_alerts.iter().map(|a| a.severity).max()
    }

    pub fn bell_pending(&self) -> bool {
        self.bell_pending
    }

    pub fn clear_bell(&mut self) {
        self.bell_pending = false;
    }

    /// Format a compact status bar indicator.
    pub fn format_indicator(&self) -> String {
        if self.active_alerts.is_empty() {
            return String::new();
        }
        let count = self.active_alerts.len();
        let sev = self.highest_severity().unwrap_or(Severity::Info);
        format!("[{} {}]", count, sev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(
        name: &str,
        metric: Metric,
        threshold: f64,
        duration: u32,
        sev: Severity,
    ) -> AlertRule {
        AlertRule {
            name: name.to_string(),
            metric,
            condition: Condition::Above,
            threshold,
            duration_ticks: duration,
            severity: sev,
            enabled: true,
        }
    }

    #[test]
    fn empty_engine_no_alerts() {
        let mut engine = AlertEngine::new(vec![], 100);
        engine.evaluate(&HashMap::new());
        assert!(!engine.has_active_alerts());
        assert!(engine.active_alerts().is_empty());
    }

    #[test]
    fn rule_fires_after_duration() {
        let rule = make_rule("High CPU", Metric::CpuTotal, 90.0, 3, Severity::Warning);
        let mut engine = AlertEngine::new(vec![rule], 100);

        let mut metrics = HashMap::new();
        metrics.insert(Metric::CpuTotal, 95.0);

        // Ticks 1 and 2: not yet
        engine.evaluate(&metrics);
        assert!(!engine.has_active_alerts());
        engine.evaluate(&metrics);
        assert!(!engine.has_active_alerts());

        // Tick 3: fires
        engine.evaluate(&metrics);
        assert!(engine.has_active_alerts());
        assert_eq!(engine.active_alerts().len(), 1);
        assert_eq!(engine.active_alerts()[0].rule_name, "High CPU");
    }

    #[test]
    fn rule_does_not_fire_before_duration() {
        let rule = make_rule("High CPU", Metric::CpuTotal, 90.0, 5, Severity::Warning);
        let mut engine = AlertEngine::new(vec![rule], 100);

        let mut metrics = HashMap::new();
        metrics.insert(Metric::CpuTotal, 95.0);

        for _ in 0..4 {
            engine.evaluate(&metrics);
        }
        assert!(!engine.has_active_alerts());
    }

    #[test]
    fn rule_resets_on_clear() {
        let rule = make_rule("High CPU", Metric::CpuTotal, 90.0, 3, Severity::Warning);
        let mut engine = AlertEngine::new(vec![rule], 100);

        let mut high = HashMap::new();
        high.insert(Metric::CpuTotal, 95.0);
        let mut low = HashMap::new();
        low.insert(Metric::CpuTotal, 50.0);

        // Build up 2 ticks
        engine.evaluate(&high);
        engine.evaluate(&high);
        // Clear condition
        engine.evaluate(&low);
        // Start over — need 3 more ticks
        engine.evaluate(&high);
        engine.evaluate(&high);
        assert!(!engine.has_active_alerts());
        engine.evaluate(&high);
        assert!(engine.has_active_alerts());
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Critical > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn format_indicator_empty() {
        let engine = AlertEngine::new(vec![], 100);
        assert_eq!(engine.format_indicator(), "");
    }

    #[test]
    fn format_indicator_shows_count() {
        let rules = vec![
            make_rule("r1", Metric::CpuTotal, 90.0, 1, Severity::Warning),
            make_rule("r2", Metric::MemoryPercent, 80.0, 1, Severity::Critical),
        ];
        let mut engine = AlertEngine::new(rules, 100);

        let mut metrics = HashMap::new();
        metrics.insert(Metric::CpuTotal, 95.0);
        metrics.insert(Metric::MemoryPercent, 85.0);

        engine.evaluate(&metrics);
        assert_eq!(engine.format_indicator(), "[2 CRIT]");
    }

    #[test]
    fn history_capped() {
        let rule = make_rule("r1", Metric::CpuTotal, 50.0, 1, Severity::Info);
        let mut engine = AlertEngine::new(vec![rule], 3);

        let mut metrics = HashMap::new();
        metrics.insert(Metric::CpuTotal, 60.0);

        // Each evaluate with a clear in between to allow new history entries
        for _ in 0..5 {
            let mut low = HashMap::new();
            low.insert(Metric::CpuTotal, 10.0);
            engine.evaluate(&low);
            engine.evaluate(&metrics);
        }
        assert!(engine.history().len() <= 3);
    }

    #[test]
    fn disabled_rules_skipped() {
        let mut rule = make_rule("disabled", Metric::CpuTotal, 50.0, 1, Severity::Critical);
        rule.enabled = false;
        let mut engine = AlertEngine::new(vec![rule], 100);

        let mut metrics = HashMap::new();
        metrics.insert(Metric::CpuTotal, 99.0);

        engine.evaluate(&metrics);
        assert!(!engine.has_active_alerts());
    }

    #[test]
    fn bell_on_critical() {
        let rule = make_rule("crit", Metric::CpuTotal, 50.0, 1, Severity::Critical);
        let mut engine = AlertEngine::new(vec![rule], 100);

        let mut metrics = HashMap::new();
        metrics.insert(Metric::CpuTotal, 99.0);

        engine.evaluate(&metrics);
        assert!(engine.bell_pending());
        engine.clear_bell();
        assert!(!engine.bell_pending());
    }
}
