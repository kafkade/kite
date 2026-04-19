//! Prometheus metrics exporter.
//!
//! When the `prometheus` feature is enabled, this module provides an embedded
//! HTTP server that exposes system metrics at `/metrics` in the Prometheus text
//! exposition format.  The server is optional, off by default, and configured
//! via `[prometheus]` in the TOML config.

// ── Feature-gated implementation ────────────────────────────────────────────

#[cfg(feature = "prometheus")]
mod inner {
    use std::io::Write;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI64, AtomicU64};

    use prometheus_client::encoding::text::encode;
    use prometheus_client::metrics::family::Family;
    use prometheus_client::metrics::gauge::Gauge;
    use prometheus_client::registry::Registry;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;
    use tokio::sync::RwLock;

    use crate::app::App;
    use crate::config::settings::PrometheusConfig;

    // ── Snapshot ────────────────────────────────────────────────────────────

    /// A point-in-time snapshot of all numeric metrics, safe to share across
    /// threads via `Arc<RwLock<…>>`.
    #[derive(Debug, Clone)]
    pub struct MetricsSnapshot {
        // CPU
        pub cpu_total_percent: f64,
        pub per_core_percent: Vec<f64>,

        // Memory
        pub mem_used_bytes: u64,
        pub mem_total_bytes: u64,
        pub swap_used_bytes: u64,
        pub swap_total_bytes: u64,

        // Disks
        pub disks: Vec<DiskSnapshot>,

        // Network
        pub net_interfaces: Vec<NetSnapshot>,

        // GPU
        pub gpus: Vec<GpuSnapshot>,

        // Sensors
        pub cpu_temp: Option<f32>,
    }

    #[derive(Debug, Clone)]
    pub struct DiskSnapshot {
        pub mount_point: String,
        pub used_bytes: u64,
        pub total_bytes: u64,
    }

    #[derive(Debug, Clone)]
    pub struct NetSnapshot {
        pub name: String,
        pub rx_bytes: u64,
        pub tx_bytes: u64,
    }

    #[derive(Debug, Clone)]
    pub struct GpuSnapshot {
        pub index: u32,
        pub name: String,
        pub utilization_percent: Option<u32>,
        pub vram_used_bytes: Option<u64>,
        pub vram_total_bytes: Option<u64>,
        pub temperature: Option<u32>,
    }

    impl Default for MetricsSnapshot {
        fn default() -> Self {
            Self {
                cpu_total_percent: 0.0,
                per_core_percent: Vec::new(),
                mem_used_bytes: 0,
                mem_total_bytes: 0,
                swap_used_bytes: 0,
                swap_total_bytes: 0,
                disks: Vec::new(),
                net_interfaces: Vec::new(),
                gpus: Vec::new(),
                cpu_temp: None,
            }
        }
    }

    /// Capture a snapshot from the current `App` state.
    pub fn collect_snapshot(app: &App) -> MetricsSnapshot {
        let disks = app
            .disk
            .disks()
            .iter()
            .map(|d| DiskSnapshot {
                mount_point: d.mount_point.clone(),
                used_bytes: d.used_bytes,
                total_bytes: d.total_bytes,
            })
            .collect();

        let net_interfaces = app
            .net
            .interfaces()
            .iter()
            .map(|n| NetSnapshot {
                name: n.name.clone(),
                rx_bytes: n.total_rx,
                tx_bytes: n.total_tx,
            })
            .collect();

        let gpus = app
            .gpu
            .devices()
            .iter()
            .map(|g| GpuSnapshot {
                index: g.index,
                name: g.name.clone(),
                utilization_percent: g.utilization_gpu,
                vram_used_bytes: g.vram_used,
                vram_total_bytes: g.vram_total,
                temperature: g.temperature,
            })
            .collect();

        MetricsSnapshot {
            cpu_total_percent: app.cpu.total_usage(),
            per_core_percent: app.cpu.per_core_usage().to_vec(),
            mem_used_bytes: app.mem.used_ram(),
            mem_total_bytes: app.mem.total_ram(),
            swap_used_bytes: app.mem.swap_used(),
            swap_total_bytes: app.mem.swap_total(),
            disks,
            net_interfaces,
            gpus,
            cpu_temp: app.sensor.cpu_temp(),
        }
    }

    // ── Label types ─────────────────────────────────────────────────────────

    /// Label set for per-core CPU metrics.
    #[derive(Clone, Debug, Hash, PartialEq, Eq, prometheus_client::encoding::EncodeLabelSet)]
    struct CoreLabel {
        core: String,
    }

    /// Label set for per-disk metrics.
    #[derive(Clone, Debug, Hash, PartialEq, Eq, prometheus_client::encoding::EncodeLabelSet)]
    struct DiskLabel {
        mount_point: String,
    }

    /// Label set for per-interface network metrics.
    #[derive(Clone, Debug, Hash, PartialEq, Eq, prometheus_client::encoding::EncodeLabelSet)]
    struct InterfaceLabel {
        interface: String,
    }

    /// Label set for per-GPU metrics.
    #[derive(Clone, Debug, Hash, PartialEq, Eq, prometheus_client::encoding::EncodeLabelSet)]
    struct GpuLabel {
        gpu: String,
    }

    // ── Registry builder ────────────────────────────────────────────────────

    /// Build a `Registry` populated with the values from a `MetricsSnapshot`.
    fn build_registry(snap: &MetricsSnapshot) -> Registry {
        let mut registry = Registry::default();

        // CPU total
        let cpu_total = Gauge::<f64, AtomicU64>::default();
        cpu_total.set(snap.cpu_total_percent);
        registry.register("kite_cpu_usage_percent", "Total CPU usage", cpu_total);

        // CPU per-core
        let cpu_core: Family<CoreLabel, Gauge<f64, AtomicU64>> = Family::default();
        for (i, &usage) in snap.per_core_percent.iter().enumerate() {
            cpu_core
                .get_or_create(&CoreLabel {
                    core: i.to_string(),
                })
                .set(usage);
        }
        registry.register(
            "kite_cpu_core_usage_percent",
            "Per-core CPU usage",
            cpu_core,
        );

        // Memory
        let mem_used = Gauge::<i64, AtomicI64>::default();
        mem_used.set(snap.mem_used_bytes as i64);
        registry.register("kite_memory_used_bytes", "Used RAM in bytes", mem_used);

        let mem_total = Gauge::<i64, AtomicI64>::default();
        mem_total.set(snap.mem_total_bytes as i64);
        registry.register("kite_memory_total_bytes", "Total RAM in bytes", mem_total);

        // Swap
        let swap_used = Gauge::<i64, AtomicI64>::default();
        swap_used.set(snap.swap_used_bytes as i64);
        registry.register("kite_swap_used_bytes", "Used swap in bytes", swap_used);

        let swap_total = Gauge::<i64, AtomicI64>::default();
        swap_total.set(snap.swap_total_bytes as i64);
        registry.register("kite_swap_total_bytes", "Total swap in bytes", swap_total);

        // Disks
        let disk_used: Family<DiskLabel, Gauge<i64, AtomicI64>> = Family::default();
        let disk_total: Family<DiskLabel, Gauge<i64, AtomicI64>> = Family::default();
        for d in &snap.disks {
            let label = DiskLabel {
                mount_point: d.mount_point.clone(),
            };
            disk_used.get_or_create(&label).set(d.used_bytes as i64);
            disk_total.get_or_create(&label).set(d.total_bytes as i64);
        }
        registry.register("kite_disk_used_bytes", "Disk used bytes", disk_used);
        registry.register("kite_disk_total_bytes", "Disk total bytes", disk_total);

        // Network
        let net_rx: Family<InterfaceLabel, Gauge<i64, AtomicI64>> = Family::default();
        let net_tx: Family<InterfaceLabel, Gauge<i64, AtomicI64>> = Family::default();
        for n in &snap.net_interfaces {
            let label = InterfaceLabel {
                interface: n.name.clone(),
            };
            net_rx.get_or_create(&label).set(n.rx_bytes as i64);
            net_tx.get_or_create(&label).set(n.tx_bytes as i64);
        }
        registry.register(
            "kite_network_rx_bytes_total",
            "Network received bytes",
            net_rx,
        );
        registry.register(
            "kite_network_tx_bytes_total",
            "Network transmitted bytes",
            net_tx,
        );

        // GPU
        let gpu_util: Family<GpuLabel, Gauge<f64, AtomicU64>> = Family::default();
        let gpu_vram_used: Family<GpuLabel, Gauge<i64, AtomicI64>> = Family::default();
        let gpu_vram_total: Family<GpuLabel, Gauge<i64, AtomicI64>> = Family::default();
        let gpu_temp: Family<GpuLabel, Gauge<f64, AtomicU64>> = Family::default();

        for g in &snap.gpus {
            let label = GpuLabel {
                gpu: g.name.clone(),
            };
            if let Some(util) = g.utilization_percent {
                gpu_util.get_or_create(&label).set(util as f64);
            }
            if let Some(used) = g.vram_used_bytes {
                gpu_vram_used.get_or_create(&label).set(used as i64);
            }
            if let Some(total) = g.vram_total_bytes {
                gpu_vram_total.get_or_create(&label).set(total as i64);
            }
            if let Some(temp) = g.temperature {
                gpu_temp.get_or_create(&label).set(temp as f64);
            }
        }

        if !snap.gpus.is_empty() {
            registry.register(
                "kite_gpu_utilization_percent",
                "GPU utilization percent",
                gpu_util,
            );
            registry.register(
                "kite_gpu_vram_used_bytes",
                "GPU VRAM used bytes",
                gpu_vram_used,
            );
            registry.register(
                "kite_gpu_vram_total_bytes",
                "GPU VRAM total bytes",
                gpu_vram_total,
            );
            registry.register(
                "kite_gpu_temperature_celsius",
                "GPU temperature in Celsius",
                gpu_temp,
            );
        }

        // CPU temperature
        if let Some(temp) = snap.cpu_temp {
            let cpu_temp_gauge = Gauge::<f64, AtomicU64>::default();
            cpu_temp_gauge.set(temp as f64);
            registry.register(
                "kite_cpu_temperature_celsius",
                "CPU temperature in Celsius",
                cpu_temp_gauge,
            );
        }

        registry
    }

    /// Encode a `MetricsSnapshot` into Prometheus text exposition format.
    pub fn encode_metrics(snap: &MetricsSnapshot) -> String {
        let registry = build_registry(snap);
        let mut buf = String::new();
        encode(&mut buf, &registry).expect("encoding to String cannot fail");
        buf
    }

    // ── HTTP Server ─────────────────────────────────────────────────────────

    /// Parse the first request line and all headers from a raw HTTP request.
    /// Returns `(method, path, headers_map)`.
    fn parse_request(raw: &str) -> Option<(&str, &str, std::collections::HashMap<&str, &str>)> {
        let mut lines = raw.lines();
        let first = lines.next()?;
        let mut parts = first.split_whitespace();
        let method = parts.next()?;
        let path = parts.next()?;

        let mut headers = std::collections::HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim(), value.trim());
            }
        }

        Some((method, path, headers))
    }

    /// Validate the bearer token in the `Authorization` header.
    fn check_auth(headers: &std::collections::HashMap<&str, &str>, expected: &str) -> bool {
        headers
            .get("Authorization")
            .map(|v| *v == format!("Bearer {expected}"))
            .unwrap_or(false)
    }

    /// Build an HTTP response with the given status, content-type, and body.
    fn http_response(status: u16, reason: &str, content_type: &str, body: &str) -> Vec<u8> {
        let mut resp = Vec::new();
        write!(
            resp,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .expect("writing to Vec cannot fail");
        resp
    }

    /// Start the Prometheus HTTP metrics server.
    ///
    /// This function runs until the `shutdown` future resolves (i.e., the
    /// application is quitting).  It is designed to be spawned as a background
    /// tokio task from `main`.
    pub async fn serve(
        config: PrometheusConfig,
        snapshot: Arc<RwLock<MetricsSnapshot>>,
        shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> anyhow::Result<()> {
        let addr = format!("{}:{}", config.bind_address, config.port);
        let listener = TcpListener::bind(&addr).await?;

        loop {
            let mut shutdown_rx = shutdown.clone();
            let accept = tokio::select! {
                result = listener.accept() => result,
                _ = shutdown_rx.changed() => break,
            };

            let (stream, _peer) = match accept {
                Ok(conn) => conn,
                Err(_) => continue,
            };

            let snap = Arc::clone(&snapshot);
            let auth_token = config.auth_token.clone();

            tokio::spawn(async move {
                let _ = handle_connection(stream, snap, auth_token.as_deref()).await;
            });
        }

        Ok(())
    }

    async fn handle_connection(
        stream: tokio::net::TcpStream,
        snapshot: Arc<RwLock<MetricsSnapshot>>,
        auth_token: Option<&str>,
    ) -> anyhow::Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut request_raw = String::new();

        // Read headers (lines until an empty line).
        loop {
            let mut line = String::new();
            let n = buf_reader.read_line(&mut line).await?;
            if n == 0 {
                return Ok(());
            }
            request_raw.push_str(&line);
            if line == "\r\n" || line == "\n" {
                break;
            }
        }

        let resp = match parse_request(&request_raw) {
            None => http_response(400, "Bad Request", "text/plain", "Bad Request\n"),
            Some((method, path, headers)) => {
                if method != "GET" {
                    http_response(
                        405,
                        "Method Not Allowed",
                        "text/plain",
                        "Method Not Allowed\n",
                    )
                } else if path != "/metrics" {
                    http_response(404, "Not Found", "text/plain", "Not Found\n")
                } else if let Some(expected) = auth_token {
                    if !check_auth(&headers, expected) {
                        http_response(401, "Unauthorized", "text/plain", "Unauthorized\n")
                    } else {
                        let snap = snapshot.read().await;
                        let body = encode_metrics(&snap);
                        http_response(200, "OK", "text/plain; version=0.0.4; charset=utf-8", &body)
                    }
                } else {
                    let snap = snapshot.read().await;
                    let body = encode_metrics(&snap);
                    http_response(200, "OK", "text/plain; version=0.0.4; charset=utf-8", &body)
                }
            }
        };

        writer.write_all(&resp).await?;
        writer.shutdown().await?;
        Ok(())
    }

    // ── Tests ───────────────────────────────────────────────────────────────

    #[cfg(test)]
    mod tests {
        use super::*;

        fn sample_snapshot() -> MetricsSnapshot {
            MetricsSnapshot {
                cpu_total_percent: 42.5,
                per_core_percent: vec![30.0, 55.0],
                mem_used_bytes: 4_000_000_000,
                mem_total_bytes: 16_000_000_000,
                swap_used_bytes: 500_000_000,
                swap_total_bytes: 8_000_000_000,
                disks: vec![DiskSnapshot {
                    mount_point: "/".to_string(),
                    used_bytes: 100_000_000_000,
                    total_bytes: 500_000_000_000,
                }],
                net_interfaces: vec![NetSnapshot {
                    name: "eth0".to_string(),
                    rx_bytes: 123456,
                    tx_bytes: 789012,
                }],
                gpus: vec![GpuSnapshot {
                    index: 0,
                    name: "RTX 4090".to_string(),
                    utilization_percent: Some(75),
                    vram_used_bytes: Some(8_000_000_000),
                    vram_total_bytes: Some(24_000_000_000),
                    temperature: Some(72),
                }],
                cpu_temp: Some(65.0),
            }
        }

        #[test]
        fn snapshot_default_is_zero() {
            let snap = MetricsSnapshot::default();
            assert_eq!(snap.cpu_total_percent, 0.0);
            assert!(snap.per_core_percent.is_empty());
            assert_eq!(snap.mem_used_bytes, 0);
            assert_eq!(snap.mem_total_bytes, 0);
            assert!(snap.disks.is_empty());
            assert!(snap.net_interfaces.is_empty());
            assert!(snap.gpus.is_empty());
            assert!(snap.cpu_temp.is_none());
        }

        #[test]
        fn encode_metrics_contains_expected_names() {
            let snap = sample_snapshot();
            let output = encode_metrics(&snap);

            // CPU
            assert!(output.contains("kite_cpu_usage_percent"));
            assert!(output.contains("kite_cpu_core_usage_percent"));
            // Memory
            assert!(output.contains("kite_memory_used_bytes"));
            assert!(output.contains("kite_memory_total_bytes"));
            // Swap
            assert!(output.contains("kite_swap_used_bytes"));
            assert!(output.contains("kite_swap_total_bytes"));
            // Disk
            assert!(output.contains("kite_disk_used_bytes"));
            assert!(output.contains("kite_disk_total_bytes"));
            // Network
            assert!(output.contains("kite_network_rx_bytes_total"));
            assert!(output.contains("kite_network_tx_bytes_total"));
            // GPU
            assert!(output.contains("kite_gpu_utilization_percent"));
            assert!(output.contains("kite_gpu_vram_used_bytes"));
            assert!(output.contains("kite_gpu_vram_total_bytes"));
            assert!(output.contains("kite_gpu_temperature_celsius"));
            // CPU temp
            assert!(output.contains("kite_cpu_temperature_celsius"));
        }

        #[test]
        fn encode_metrics_contains_label_values() {
            let snap = sample_snapshot();
            let output = encode_metrics(&snap);

            // Per-core labels
            assert!(output.contains("core=\"0\""));
            assert!(output.contains("core=\"1\""));
            // Disk labels
            assert!(output.contains("mount_point=\"/\""));
            // Net labels
            assert!(output.contains("interface=\"eth0\""));
            // GPU labels
            assert!(output.contains("gpu=\"RTX 4090\""));
        }

        #[test]
        fn encode_metrics_contains_values() {
            let snap = sample_snapshot();
            let output = encode_metrics(&snap);

            assert!(output.contains("42.5"));
            assert!(output.contains("4000000000"));
            assert!(output.contains("16000000000"));
        }

        #[test]
        fn encode_empty_snapshot_does_not_include_gpu() {
            let snap = MetricsSnapshot::default();
            let output = encode_metrics(&snap);

            // GPU metrics should not appear when there are no GPUs
            assert!(!output.contains("kite_gpu_utilization_percent"));
            assert!(!output.contains("kite_gpu_vram_used_bytes"));
        }

        #[test]
        fn parse_request_valid() {
            let raw =
                "GET /metrics HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer secret\r\n\r\n";
            let (method, path, headers) = parse_request(raw).unwrap();
            assert_eq!(method, "GET");
            assert_eq!(path, "/metrics");
            assert_eq!(headers.get("Authorization"), Some(&"Bearer secret"));
        }

        #[test]
        fn parse_request_empty() {
            assert!(parse_request("").is_none());
        }

        #[test]
        fn check_auth_correct_token() {
            let mut headers = std::collections::HashMap::new();
            headers.insert("Authorization", "Bearer my-secret");
            assert!(check_auth(&headers, "my-secret"));
        }

        #[test]
        fn check_auth_wrong_token() {
            let mut headers = std::collections::HashMap::new();
            headers.insert("Authorization", "Bearer wrong");
            assert!(!check_auth(&headers, "my-secret"));
        }

        #[test]
        fn check_auth_missing_header() {
            let headers = std::collections::HashMap::new();
            assert!(!check_auth(&headers, "my-secret"));
        }

        #[test]
        fn http_response_format() {
            let resp = http_response(200, "OK", "text/plain", "hello");
            let text = String::from_utf8(resp).unwrap();
            assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
            assert!(text.contains("Content-Length: 5"));
            assert!(text.ends_with("hello"));
        }

        #[tokio::test]
        async fn server_responds_to_metrics_request() {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            let snap = Arc::new(RwLock::new(sample_snapshot()));
            let _config = PrometheusConfig {
                enabled: true,
                port: 0, // will be overridden by listener
                bind_address: "127.0.0.1".to_string(),
                auth_token: None,
            };

            // Bind to a random port
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let snap_clone = Arc::clone(&snap);
            let handle = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                handle_connection(stream, snap_clone, None).await.unwrap();
            });

            // Connect and send request
            let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
            client
                .write_all(b"GET /metrics HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await
                .unwrap();

            let mut response = String::new();
            client.read_to_string(&mut response).await.unwrap();

            assert!(response.starts_with("HTTP/1.1 200 OK"));
            assert!(response.contains("kite_cpu_usage_percent"));
            handle.await.unwrap();
        }

        #[tokio::test]
        async fn server_returns_401_without_token() {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            let snap = Arc::new(RwLock::new(sample_snapshot()));

            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let snap_clone = Arc::clone(&snap);
            let handle = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                handle_connection(stream, snap_clone, Some("secret123"))
                    .await
                    .unwrap();
            });

            let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
            client
                .write_all(b"GET /metrics HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await
                .unwrap();

            let mut response = String::new();
            client.read_to_string(&mut response).await.unwrap();

            assert!(response.contains("401 Unauthorized"));
            handle.await.unwrap();
        }

        #[tokio::test]
        async fn server_returns_404_for_wrong_path() {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            let snap = Arc::new(RwLock::new(MetricsSnapshot::default()));

            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let snap_clone = Arc::clone(&snap);
            let handle = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                handle_connection(stream, snap_clone, None).await.unwrap();
            });

            let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();
            client
                .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await
                .unwrap();

            let mut response = String::new();
            client.read_to_string(&mut response).await.unwrap();

            assert!(response.contains("404 Not Found"));
            handle.await.unwrap();
        }
    }
}

// ── Public re-exports (feature ON) ──────────────────────────────────────────

#[cfg(feature = "prometheus")]
pub use inner::*;

// ── Stubs (feature OFF) ─────────────────────────────────────────────────────

/// When the `prometheus` feature is disabled, `MetricsSnapshot` is a no-op
/// empty struct so that main.rs can compile unconditionally.
#[cfg(not(feature = "prometheus"))]
#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot;

/// Stub: collect a (meaningless) snapshot when prometheus is disabled.
#[cfg(not(feature = "prometheus"))]
pub fn collect_snapshot(_app: &crate::app::App) -> MetricsSnapshot {
    MetricsSnapshot
}
