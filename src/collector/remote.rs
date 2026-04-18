#[allow(unused_imports)]
use std::time::{Duration, Instant};

#[allow(unused_imports)]
use anyhow::Result;

use crate::collector::Collector;
use crate::config::settings::RemoteConfig;
use crate::util::ring_buffer::RingBuffer;

/// Connection state of a remote machine.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Connecting => write!(f, "Connecting"),
            ConnectionState::Disconnected => write!(f, "Disconnected"),
            ConnectionState::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}

/// Snapshot of a remote machine's system metrics.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RemoteSnapshot {
    pub name: String,
    pub host: String,
    pub state: ConnectionState,
    pub latency_ms: Option<u64>,
    pub cpu_usage: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub net_rx_rate: u64,
    pub net_tx_rate: u64,
    pub uptime_secs: u64,
    pub cpu_history: RingBuffer<f64>,
}

impl RemoteSnapshot {
    pub fn new(name: &str, host: &str, history_capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            host: host.to_string(),
            state: ConnectionState::Disconnected,
            latency_ms: None,
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            disk_used: 0,
            disk_total: 0,
            net_rx_rate: 0,
            net_tx_rate: 0,
            uptime_secs: 0,
            cpu_history: RingBuffer::new(history_capacity),
        }
    }

    pub fn memory_percent(&self) -> f64 {
        if self.memory_total == 0 {
            0.0
        } else {
            (self.memory_used as f64 / self.memory_total as f64) * 100.0
        }
    }

    pub fn disk_percent(&self) -> f64 {
        if self.disk_total == 0 {
            0.0
        } else {
            (self.disk_used as f64 / self.disk_total as f64) * 100.0
        }
    }
}

// ── Agent script ────────────────────────────────────────────────────────────

/// Shell script executed on remote hosts to gather metrics.
/// Outputs structured data separated by markers for reliable parsing.
#[allow(dead_code)]
const AGENT_SCRIPT: &str = r#"LC_ALL=C; export LC_ALL
echo '---KITE-BEGIN---'
head -1 /proc/stat
echo '---KITE-SEP---'
grep -E '^(MemTotal|MemFree|MemAvailable|Buffers|Cached|SwapTotal|SwapFree):' /proc/meminfo
echo '---KITE-SEP---'
df -B1 --total 2>/dev/null | tail -1 || echo 'total 0 0 0 0 0% /'
echo '---KITE-SEP---'
cat /proc/net/dev
echo '---KITE-SEP---'
cat /proc/uptime
echo '---KITE-END---'"#;

/// Previous sample data for delta calculations.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct PrevSample {
    cpu_idle: u64,
    cpu_total: u64,
    net_rx: u64,
    net_tx: u64,
    timestamp: Option<Instant>,
}

// ── Feature-gated: SSH ON ───────────────────────────────────────────────────

#[cfg(feature = "ssh")]
mod ssh_impl {
    use super::*;
    use std::sync::Arc;

    /// SSH client handler for russh.
    pub(super) struct SshHandler;

    #[async_trait::async_trait]
    impl russh::client::Handler for SshHandler {
        type Error = russh::Error;

        async fn check_server_key(
            &mut self,
            _server_public_key: &ssh_key::PublicKey,
        ) -> std::result::Result<bool, Self::Error> {
            // Accept all host keys — see #49 for known_hosts verification.
            Ok(true)
        }
    }

    /// Per-remote SSH connection state.
    pub(super) struct SshConnection {
        pub config: RemoteConfig,
        pub handle: Option<russh::client::Handle<SshHandler>>,
        pub prev: PrevSample,
        pub reconnect_at: Option<Instant>,
        pub backoff: Duration,
    }

    impl SshConnection {
        pub fn new(config: RemoteConfig) -> Self {
            Self {
                config,
                handle: None,
                prev: PrevSample::default(),
                reconnect_at: None,
                backoff: Duration::from_secs(1),
            }
        }

        /// Attempt to establish an SSH connection.
        pub async fn connect(&mut self) -> Result<()> {
            let ssh_config = russh::client::Config {
                inactivity_timeout: Some(Duration::from_secs(30)),
                ..Default::default()
            };

            let handler = SshHandler;
            let mut session = tokio::time::timeout(
                Duration::from_secs(10),
                russh::client::connect(
                    Arc::new(ssh_config),
                    (self.config.host.as_str(), self.config.port),
                    handler,
                ),
            )
            .await
            .map_err(|_| anyhow::anyhow!("SSH connect timeout"))??;

            // Authenticate with key file or try default key locations
            let key_path = if let Some(ref kp) = self.config.key {
                Some(shellexpand_path(kp))
            } else {
                find_default_key()
            };

            let authenticated = if let Some(path) = key_path {
                let key = russh::keys::load_secret_key(&path, None)
                    .map_err(|e| anyhow::anyhow!("Failed to load key {}: {}", path, e))?;
                session
                    .authenticate_publickey(&self.config.user, Arc::new(key))
                    .await?
            } else {
                return Err(anyhow::anyhow!(
                    "No key file found — specify key in [[remotes]] config"
                ));
            };

            if !authenticated {
                return Err(anyhow::anyhow!("SSH authentication failed"));
            }

            self.handle = Some(session);
            self.backoff = Duration::from_secs(1);
            self.reconnect_at = None;
            Ok(())
        }

        /// Execute the agent script on the remote and return raw output.
        pub async fn exec_agent(&mut self) -> Result<String> {
            let session = self
                .handle
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("Not connected"))?;

            let mut channel =
                tokio::time::timeout(Duration::from_secs(5), session.channel_open_session())
                    .await
                    .map_err(|_| anyhow::anyhow!("Channel open timeout"))??;

            channel.exec(true, AGENT_SCRIPT).await?;

            let mut output = String::new();
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                if Instant::now() > deadline {
                    return Err(anyhow::anyhow!("Command execution timeout"));
                }
                let msg = tokio::time::timeout(Duration::from_secs(3), channel.wait())
                    .await
                    .map_err(|_| anyhow::anyhow!("Read timeout"))?;

                match msg {
                    Some(russh::ChannelMsg::Data { ref data }) => {
                        output.push_str(&String::from_utf8_lossy(data));
                    }
                    Some(russh::ChannelMsg::ExitStatus { .. }) | None => break,
                    _ => {}
                }
            }
            Ok(output)
        }

        /// Schedule a reconnection attempt with exponential backoff.
        pub fn schedule_reconnect(&mut self) {
            self.handle = None;
            self.reconnect_at = Some(Instant::now() + self.backoff);
            // Exponential backoff, capped at 60 seconds
            self.backoff = (self.backoff * 2).min(Duration::from_secs(60));
        }

        pub fn should_reconnect(&self) -> bool {
            match self.reconnect_at {
                Some(t) => Instant::now() >= t,
                None => self.handle.is_none(),
            }
        }
    }

    /// Expand ~ to home directory in key paths.
    fn shellexpand_path(path: &str) -> String {
        if let Some(rest) = path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(rest).to_string_lossy().to_string();
            }
        }
        path.to_string()
    }

    /// Try to find a default SSH key in ~/.ssh/
    fn find_default_key() -> Option<String> {
        let ssh_dir = dirs::home_dir()?.join(".ssh");
        for name in &["id_ed25519", "id_rsa", "id_ecdsa"] {
            let path = ssh_dir.join(name);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
        None
    }
}

// ── Parsing ─────────────────────────────────────────────────────────────────

/// Parsed metrics from the agent script output.
#[derive(Debug, Default)]
#[allow(dead_code)]
struct ParsedMetrics {
    cpu_idle: u64,
    cpu_total: u64,
    mem_total_kb: u64,
    mem_available_kb: u64,
    disk_used: u64,
    disk_total: u64,
    net_rx: u64,
    net_tx: u64,
    uptime_secs: u64,
}

#[allow(dead_code)]
fn parse_agent_output(output: &str) -> Option<ParsedMetrics> {
    let begin = output.find("---KITE-BEGIN---")?;
    let end = output.find("---KITE-END---")?;
    let body = &output[begin + "---KITE-BEGIN---".len()..end];
    let sections: Vec<&str> = body.split("---KITE-SEP---").collect();
    if sections.len() < 5 {
        return None;
    }

    let mut metrics = ParsedMetrics::default();

    // Section 0: /proc/stat first line — "cpu user nice system idle ..."
    parse_cpu_stat(sections[0].trim(), &mut metrics);

    // Section 1: /proc/meminfo fields
    parse_meminfo(sections[1].trim(), &mut metrics);

    // Section 2: df total line
    parse_df(sections[2].trim(), &mut metrics);

    // Section 3: /proc/net/dev
    parse_netdev(sections[3].trim(), &mut metrics);

    // Section 4: /proc/uptime
    parse_uptime(sections[4].trim(), &mut metrics);

    Some(metrics)
}

#[allow(dead_code)]
fn parse_cpu_stat(s: &str, m: &mut ParsedMetrics) {
    // "cpu  user nice system idle iowait irq softirq steal guest guest_nice"
    let parts: Vec<u64> = s
        .split_whitespace()
        .skip(1) // skip "cpu"
        .filter_map(|v| v.parse().ok())
        .collect();
    if parts.len() >= 4 {
        m.cpu_total = parts.iter().sum();
        m.cpu_idle = parts[3]; // idle is the 4th field
    }
}

#[allow(dead_code)]
fn parse_meminfo(s: &str, m: &mut ParsedMetrics) {
    for line in s.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let val: u64 = parts[1].parse().unwrap_or(0);
            match parts[0] {
                "MemTotal:" => m.mem_total_kb = val,
                "MemAvailable:" => m.mem_available_kb = val,
                _ => {}
            }
        }
    }
}

#[allow(dead_code)]
fn parse_df(s: &str, m: &mut ParsedMetrics) {
    // "total  total_bytes  used_bytes  avail_bytes  use%  mount"
    // or with -B1: fields are in bytes
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() >= 4 {
        m.disk_total = parts[1].parse().unwrap_or(0);
        m.disk_used = parts[2].parse().unwrap_or(0);
    }
}

#[allow(dead_code)]
fn parse_netdev(s: &str, m: &mut ParsedMetrics) {
    // Skip header lines (contain |), sum all interfaces except lo
    for line in s.lines() {
        if line.contains('|') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            let iface = parts[0].trim_end_matches(':');
            if iface == "lo" {
                continue;
            }
            m.net_rx += parts[1].parse::<u64>().unwrap_or(0);
            m.net_tx += parts[9].parse::<u64>().unwrap_or(0);
        }
    }
}

#[allow(dead_code)]
fn parse_uptime(s: &str, m: &mut ParsedMetrics) {
    // "uptime_seconds idle_seconds"
    if let Some(first) = s.split_whitespace().next() {
        if let Ok(val) = first.parse::<f64>() {
            m.uptime_secs = val as u64;
        }
    }
}

// ── RemoteCollector ─────────────────────────────────────────────────────────

/// Collects system metrics from remote machines via SSH.
pub struct RemoteCollector {
    #[cfg(feature = "ssh")]
    connections: Vec<ssh_impl::SshConnection>,
    snapshots: Vec<RemoteSnapshot>,
    history_capacity: usize,
}

// ── Feature-gated implementation: SSH ON ────────────────────────────────────

#[cfg(feature = "ssh")]
impl RemoteCollector {
    pub fn new(configs: &[RemoteConfig], history_capacity: usize) -> Self {
        let connections: Vec<ssh_impl::SshConnection> = configs
            .iter()
            .map(|c| ssh_impl::SshConnection::new(c.clone()))
            .collect();
        let snapshots: Vec<RemoteSnapshot> = configs
            .iter()
            .map(|c| RemoteSnapshot::new(&c.name, &c.host, history_capacity))
            .collect();
        Self {
            connections,
            snapshots,
            history_capacity,
        }
    }

    async fn collect_async(&mut self) {
        for i in 0..self.connections.len() {
            let conn = &mut self.connections[i];
            let snap = &mut self.snapshots[i];

            // Try to connect if needed
            if conn.should_reconnect() {
                snap.state = ConnectionState::Connecting;
                match conn.connect().await {
                    Ok(()) => {
                        snap.state = ConnectionState::Connected;
                    }
                    Err(e) => {
                        snap.state = ConnectionState::Error(e.to_string());
                        conn.schedule_reconnect();
                        continue;
                    }
                }
            }

            if conn.handle.is_none() {
                continue;
            }

            // Execute agent script and measure latency
            let start = Instant::now();
            match conn.exec_agent().await {
                Ok(output) => {
                    let latency = start.elapsed().as_millis() as u64;
                    snap.latency_ms = Some(latency);
                    snap.state = ConnectionState::Connected;

                    if let Some(metrics) = parse_agent_output(&output) {
                        // Compute CPU% from deltas
                        let prev = &conn.prev;
                        if prev.cpu_total > 0 {
                            let total_delta = metrics.cpu_total.saturating_sub(prev.cpu_total);
                            let idle_delta = metrics.cpu_idle.saturating_sub(prev.cpu_idle);
                            if total_delta > 0 {
                                snap.cpu_usage = ((total_delta - idle_delta) as f64
                                    / total_delta as f64)
                                    * 100.0;
                            }
                        }
                        snap.cpu_history.push(snap.cpu_usage);

                        // Memory (kB to bytes)
                        snap.memory_total = metrics.mem_total_kb * 1024;
                        let mem_available = metrics.mem_available_kb * 1024;
                        snap.memory_used = snap.memory_total.saturating_sub(mem_available);

                        // Disk
                        snap.disk_total = metrics.disk_total;
                        snap.disk_used = metrics.disk_used;

                        // Network rates from deltas
                        if let Some(prev_ts) = prev.timestamp {
                            let elapsed = start.duration_since(prev_ts).as_secs_f64();
                            if elapsed > 0.0 {
                                snap.net_rx_rate =
                                    ((metrics.net_rx.saturating_sub(prev.net_rx)) as f64 / elapsed)
                                        as u64;
                                snap.net_tx_rate =
                                    ((metrics.net_tx.saturating_sub(prev.net_tx)) as f64 / elapsed)
                                        as u64;
                            }
                        }

                        snap.uptime_secs = metrics.uptime_secs;

                        // Store current sample for next delta
                        conn.prev = PrevSample {
                            cpu_idle: metrics.cpu_idle,
                            cpu_total: metrics.cpu_total,
                            net_rx: metrics.net_rx,
                            net_tx: metrics.net_tx,
                            timestamp: Some(start),
                        };
                    }
                }
                Err(_) => {
                    snap.state = ConnectionState::Disconnected;
                    snap.latency_ms = None;
                    conn.schedule_reconnect();
                }
            }
        }
    }
}

#[cfg(feature = "ssh")]
impl Collector for RemoteCollector {
    fn collect(&mut self) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.collect_async())
        });
        Ok(())
    }
}

// ── Feature-gated implementation: SSH OFF ───────────────────────────────────

#[cfg(not(feature = "ssh"))]
impl RemoteCollector {
    pub fn new(configs: &[RemoteConfig], history_capacity: usize) -> Self {
        let snapshots: Vec<RemoteSnapshot> = configs
            .iter()
            .map(|c| RemoteSnapshot::new(&c.name, &c.host, history_capacity))
            .collect();
        Self {
            snapshots,
            history_capacity,
        }
    }
}

#[cfg(not(feature = "ssh"))]
impl Collector for RemoteCollector {
    fn collect(&mut self) -> Result<()> {
        Ok(())
    }
}

// ── Shared getters (always available) ───────────────────────────────────────

impl RemoteCollector {
    pub fn snapshots(&self) -> &[RemoteSnapshot] {
        &self.snapshots
    }

    pub fn remote_count(&self) -> usize {
        self.snapshots.len()
    }

    pub fn connected_count(&self) -> usize {
        self.snapshots
            .iter()
            .filter(|s| s.state == ConnectionState::Connected)
            .count()
    }

    pub fn has_remotes(&self) -> bool {
        !self.snapshots.is_empty()
    }

    #[allow(dead_code)]
    pub fn history_capacity(&self) -> usize {
        self.history_capacity
    }

    /// Aggregate CPU usage across all connected remotes.
    pub fn aggregate_cpu(&self) -> Option<f64> {
        let connected: Vec<&RemoteSnapshot> = self
            .snapshots
            .iter()
            .filter(|s| s.state == ConnectionState::Connected)
            .collect();
        if connected.is_empty() {
            return None;
        }
        let sum: f64 = connected.iter().map(|s| s.cpu_usage).sum();
        Some(sum / connected.len() as f64)
    }

    /// Aggregate memory usage across all connected remotes.
    pub fn aggregate_memory(&self) -> Option<(u64, u64)> {
        let connected: Vec<&RemoteSnapshot> = self
            .snapshots
            .iter()
            .filter(|s| s.state == ConnectionState::Connected)
            .collect();
        if connected.is_empty() {
            return None;
        }
        let used: u64 = connected.iter().map(|s| s.memory_used).sum();
        let total: u64 = connected.iter().map(|s| s.memory_total).sum();
        Some((used, total))
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_state_display() {
        assert_eq!(ConnectionState::Connected.to_string(), "Connected");
        assert_eq!(ConnectionState::Connecting.to_string(), "Connecting");
        assert_eq!(ConnectionState::Disconnected.to_string(), "Disconnected");
        assert_eq!(
            ConnectionState::Error("timeout".to_string()).to_string(),
            "Error: timeout"
        );
    }

    #[test]
    fn remote_snapshot_new() {
        let snap = RemoteSnapshot::new("web-1", "10.0.1.5", 100);
        assert_eq!(snap.name, "web-1");
        assert_eq!(snap.host, "10.0.1.5");
        assert_eq!(snap.state, ConnectionState::Disconnected);
        assert!(snap.latency_ms.is_none());
        assert_eq!(snap.cpu_history.capacity(), 100);
    }

    #[test]
    fn snapshot_memory_percent() {
        let mut snap = RemoteSnapshot::new("test", "host", 10);
        snap.memory_used = 4 * 1024 * 1024 * 1024; // 4 GiB
        snap.memory_total = 16 * 1024 * 1024 * 1024; // 16 GiB
        assert!((snap.memory_percent() - 25.0).abs() < 0.01);

        snap.memory_total = 0;
        assert_eq!(snap.memory_percent(), 0.0);
    }

    #[test]
    fn snapshot_disk_percent() {
        let mut snap = RemoteSnapshot::new("test", "host", 10);
        snap.disk_used = 500_000_000_000;
        snap.disk_total = 1_000_000_000_000;
        assert!((snap.disk_percent() - 50.0).abs() < 0.01);

        snap.disk_total = 0;
        assert_eq!(snap.disk_percent(), 0.0);
    }

    #[test]
    fn parse_agent_output_valid() {
        let output = r#"---KITE-BEGIN---
cpu  1000 200 300 500 10 5 3 0 0 0
---KITE-SEP---
MemTotal:       16384000 kB
MemFree:         2000000 kB
MemAvailable:    8000000 kB
Buffers:          500000 kB
Cached:          4000000 kB
SwapTotal:       8192000 kB
SwapFree:        8000000 kB
---KITE-SEP---
total 500000000000 200000000000 300000000000 40% /
---KITE-SEP---
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 1000 10 0 0 0 0 0 0 1000 10 0 0 0 0 0 0
  eth0: 5000000 4000 0 0 0 0 0 0 3000000 3000 0 0 0 0 0 0
---KITE-SEP---
12345.67 54321.00
---KITE-END---"#;

        let metrics = parse_agent_output(output).expect("should parse");
        // cpu: total=1000+200+300+500+10+5+3=2018, idle=500
        assert_eq!(metrics.cpu_total, 2018);
        assert_eq!(metrics.cpu_idle, 500);
        assert_eq!(metrics.mem_total_kb, 16384000);
        assert_eq!(metrics.mem_available_kb, 8000000);
        assert_eq!(metrics.disk_total, 500000000000);
        assert_eq!(metrics.disk_used, 200000000000);
        assert_eq!(metrics.net_rx, 5000000); // eth0 only (lo excluded)
        assert_eq!(metrics.net_tx, 3000000);
        assert_eq!(metrics.uptime_secs, 12345);
    }

    #[test]
    fn parse_agent_output_missing_markers() {
        assert!(parse_agent_output("no markers here").is_none());
        assert!(parse_agent_output("---KITE-BEGIN---\n---KITE-END---").is_none());
    }

    #[test]
    fn parse_cpu_stat_works() {
        let mut m = ParsedMetrics::default();
        parse_cpu_stat("cpu  100 50 30 200 10 5 2 0 0 0", &mut m);
        assert_eq!(m.cpu_total, 397);
        assert_eq!(m.cpu_idle, 200);
    }

    #[test]
    fn parse_meminfo_works() {
        let mut m = ParsedMetrics::default();
        parse_meminfo(
            "MemTotal:       16384000 kB\nMemAvailable:    8000000 kB",
            &mut m,
        );
        assert_eq!(m.mem_total_kb, 16384000);
        assert_eq!(m.mem_available_kb, 8000000);
    }

    #[test]
    fn parse_netdev_skips_lo() {
        let mut m = ParsedMetrics::default();
        let input = "Inter-|   Receive\n face |bytes\n    lo: 1000 10 0 0 0 0 0 0 1000 10 0 0 0 0 0 0\n  eth0: 5000 40 0 0 0 0 0 0 3000 30 0 0 0 0 0 0";
        parse_netdev(input, &mut m);
        assert_eq!(m.net_rx, 5000);
        assert_eq!(m.net_tx, 3000);
    }

    #[test]
    fn collector_no_remotes() {
        let collector = RemoteCollector::new(&[], 300);
        assert!(!collector.has_remotes());
        assert_eq!(collector.remote_count(), 0);
        assert_eq!(collector.connected_count(), 0);
        assert!(collector.aggregate_cpu().is_none());
        assert!(collector.aggregate_memory().is_none());
    }

    #[test]
    fn collector_with_configs() {
        let configs = vec![RemoteConfig {
            name: "test-server".to_string(),
            host: "192.168.1.1".to_string(),
            port: 22,
            user: "monitor".to_string(),
            key: None,
            agent_forwarding: false,
        }];
        let collector = RemoteCollector::new(&configs, 300);
        assert!(collector.has_remotes());
        assert_eq!(collector.remote_count(), 1);
        assert_eq!(collector.connected_count(), 0); // not connected yet
        assert_eq!(collector.snapshots()[0].name, "test-server");
        assert_eq!(collector.snapshots()[0].host, "192.168.1.1");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn collect_with_no_remotes_succeeds() {
        let mut collector = RemoteCollector::new(&[], 300);
        assert!(collector.collect().is_ok());
    }
}
