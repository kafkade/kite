use std::collections::HashMap;
use std::time::Instant;

use anyhow::Result;
use sysinfo::Networks;

use crate::collector::Collector;
use crate::util::ring_buffer::RingBuffer;

/// Per-interface network statistics.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InterfaceInfo {
    pub name: String,
    pub rx_bytes_sec: f64,
    pub tx_bytes_sec: f64,
    pub total_rx: u64,
    pub total_tx: u64,
}

/// Collects network metrics: per-interface speeds and cumulative totals.
pub struct NetworkCollector {
    networks: Networks,
    interfaces: Vec<InterfaceInfo>,
    prev_counters: HashMap<String, (u64, u64)>,
    last_collect: Instant,
    total_rx_bytes_sec: f64,
    total_tx_bytes_sec: f64,
    total_rx_bytes: u64,
    total_tx_bytes: u64,
    rx_history: RingBuffer<f64>,
    tx_history: RingBuffer<f64>,
}

#[allow(dead_code)]
impl NetworkCollector {
    pub fn new(history_capacity: usize) -> Self {
        let networks = Networks::new_with_refreshed_list();

        let mut prev_counters = HashMap::new();
        for (name, data) in networks.list() {
            prev_counters.insert(name.clone(), (data.received(), data.transmitted()));
        }

        Self {
            networks,
            interfaces: Vec::new(),
            prev_counters,
            last_collect: Instant::now(),
            total_rx_bytes_sec: 0.0,
            total_tx_bytes_sec: 0.0,
            total_rx_bytes: 0,
            total_tx_bytes: 0,
            rx_history: RingBuffer::new(history_capacity),
            tx_history: RingBuffer::new(history_capacity),
        }
    }

    pub fn interfaces(&self) -> &[InterfaceInfo] {
        &self.interfaces
    }

    pub fn total_rx_bytes_sec(&self) -> f64 {
        self.total_rx_bytes_sec
    }

    pub fn total_tx_bytes_sec(&self) -> f64 {
        self.total_tx_bytes_sec
    }

    pub fn rx_history(&self) -> &RingBuffer<f64> {
        &self.rx_history
    }

    pub fn tx_history(&self) -> &RingBuffer<f64> {
        &self.tx_history
    }

    pub fn total_rx_bytes(&self) -> u64 {
        self.total_rx_bytes
    }

    pub fn total_tx_bytes(&self) -> u64 {
        self.total_tx_bytes
    }

    /// Set network data directly (used by replay mode).
    pub fn set_network_data(
        &mut self,
        interfaces: Vec<InterfaceInfo>,
        total_rx_bytes_sec: f64,
        total_tx_bytes_sec: f64,
    ) {
        self.interfaces = interfaces;
        self.total_rx_bytes_sec = total_rx_bytes_sec;
        self.total_tx_bytes_sec = total_tx_bytes_sec;
        self.rx_history.push(total_rx_bytes_sec);
        self.tx_history.push(total_tx_bytes_sec);
    }
}

impl Collector for NetworkCollector {
    fn collect(&mut self) -> Result<()> {
        self.networks.refresh(true);

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_collect).as_secs_f64();
        let elapsed = if elapsed > 0.0 { elapsed } else { 1.0 };
        self.last_collect = now;

        let mut interfaces = Vec::new();
        let mut agg_rx = 0.0_f64;
        let mut agg_tx = 0.0_f64;
        let mut session_rx = 0_u64;
        let mut session_tx = 0_u64;

        let mut new_prev = HashMap::new();

        for (name, data) in self.networks.list() {
            let cur_rx = data.received();
            let cur_tx = data.transmitted();

            let (prev_rx, prev_tx) = self
                .prev_counters
                .get(name)
                .copied()
                .unwrap_or((cur_rx, cur_tx));

            let delta_rx = cur_rx.saturating_sub(prev_rx);
            let delta_tx = cur_tx.saturating_sub(prev_tx);

            let rx_bps = delta_rx as f64 / elapsed;
            let tx_bps = delta_tx as f64 / elapsed;

            session_rx = session_rx.saturating_add(delta_rx);
            session_tx = session_tx.saturating_add(delta_tx);
            agg_rx += rx_bps;
            agg_tx += tx_bps;

            interfaces.push(InterfaceInfo {
                name: name.clone(),
                rx_bytes_sec: rx_bps,
                tx_bytes_sec: tx_bps,
                total_rx: cur_rx,
                total_tx: cur_tx,
            });

            new_prev.insert(name.clone(), (cur_rx, cur_tx));
        }

        self.prev_counters = new_prev;
        self.interfaces = interfaces;
        self.total_rx_bytes_sec = agg_rx;
        self.total_tx_bytes_sec = agg_tx;
        self.total_rx_bytes = self.total_rx_bytes.saturating_add(session_rx);
        self.total_tx_bytes = self.total_tx_bytes.saturating_add(session_tx);
        self.rx_history.push(agg_rx);
        self.tx_history.push(agg_tx);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes() {
        let collector = NetworkCollector::new(60);
        assert!(collector.interfaces().is_empty());
        assert_eq!(collector.total_rx_bytes_sec(), 0.0);
        assert_eq!(collector.total_tx_bytes_sec(), 0.0);
    }

    #[test]
    fn collect_succeeds() {
        let mut collector = NetworkCollector::new(60);
        assert!(collector.collect().is_ok());
    }

    #[test]
    fn speeds_are_non_negative() {
        let mut collector = NetworkCollector::new(60);
        collector.collect().unwrap();
        assert!(collector.total_rx_bytes_sec() >= 0.0);
        assert!(collector.total_tx_bytes_sec() >= 0.0);
        for iface in collector.interfaces() {
            assert!(iface.rx_bytes_sec >= 0.0);
            assert!(iface.tx_bytes_sec >= 0.0);
        }
    }

    #[test]
    fn history_has_entries_after_two_collects() {
        let mut collector = NetworkCollector::new(60);
        collector.collect().unwrap();
        collector.collect().unwrap();
        assert!(collector.rx_history().len() >= 2);
        assert!(collector.tx_history().len() >= 2);
    }
}
