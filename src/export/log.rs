//! File-based metrics logging.
//!
//! Writes selected system metrics to a log file on each data tick.
//! Supports JSON Lines and CSV formats, log rotation by size or time,
//! optional gzip compression of rotated files, and configurable retention.

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::Utc;
use flate2::Compression;
use flate2::write::GzEncoder;
use serde::Serialize;

use crate::app::App;
use crate::config::settings::{LogFormat, LoggingConfig};

// ---------------------------------------------------------------------------
// Serializable snapshot types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CpuSnapshot {
    total: f64,
    cores: Vec<f64>,
}

#[derive(Serialize)]
struct MemorySnapshot {
    used: u64,
    total: u64,
    swap_used: u64,
    swap_total: u64,
}

#[derive(Serialize)]
struct DiskEntry {
    mount: String,
    total_bytes: u64,
    used_bytes: u64,
    free_bytes: u64,
    usage_percent: f64,
}

#[derive(Serialize)]
struct DiskSnapshot {
    read_bytes_sec: f64,
    write_bytes_sec: f64,
    filesystems: Vec<DiskEntry>,
}

#[derive(Serialize)]
struct NetEntry {
    name: String,
    rx_bytes_sec: f64,
    tx_bytes_sec: f64,
    total_rx: u64,
    total_tx: u64,
}

#[derive(Serialize)]
struct TickRecord {
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu: Option<CpuSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<MemorySnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disk: Option<DiskSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<Vec<NetEntry>>,
}

// ---------------------------------------------------------------------------
// MetricsLogger
// ---------------------------------------------------------------------------

/// Logs system metrics to a file on each data tick.
pub struct MetricsLogger {
    config: LoggingConfig,
    log_dir: PathBuf,
    writer: Option<BufWriter<File>>,
    current_file: PathBuf,
    current_size: u64,
    rotation_start: Instant,
    csv_header_written: bool,
}

impl MetricsLogger {
    /// Create a new logger. Returns `None` if logging is disabled.
    pub fn new(config: &LoggingConfig) -> Option<Self> {
        if !config.enabled {
            return None;
        }

        let log_dir = resolve_log_dir(config);

        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!(
                "kite: failed to create log directory {}: {e}",
                log_dir.display()
            );
            return None;
        }

        let ext = match config.format {
            LogFormat::Json => "jsonl",
            LogFormat::Csv => "csv",
        };
        let current_file = log_dir.join(format!("kite-metrics.{ext}"));

        let writer = match open_append(&current_file) {
            Ok(w) => w,
            Err(e) => {
                eprintln!(
                    "kite: failed to open log file {}: {e}",
                    current_file.display()
                );
                return None;
            }
        };

        let current_size = current_file.metadata().map(|m| m.len()).unwrap_or(0);

        Some(Self {
            config: config.clone(),
            log_dir,
            writer: Some(writer),
            current_file,
            current_size,
            rotation_start: Instant::now(),
            csv_header_written: current_size > 0,
        })
    }

    /// Record one tick of metrics from the application.
    pub fn log_tick(&mut self, app: &App) {
        let record = self.build_record(app);

        let line = match self.config.format {
            LogFormat::Json => match serde_json::to_string(&record) {
                Ok(s) => s + "\n",
                Err(e) => {
                    eprintln!("kite: log serialization error: {e}");
                    return;
                }
            },
            LogFormat::Csv => self.format_csv(&record),
        };

        if let Err(e) = self.write_line(&line) {
            eprintln!("kite: log write error: {e}");
            return;
        }

        if self.should_rotate() {
            if let Err(e) = self.rotate() {
                eprintln!("kite: log rotation error: {e}");
            }
        }
    }

    // -- private helpers ---------------------------------------------------

    fn should_log(&self, metric: &str) -> bool {
        self.config.metrics.is_empty() || self.config.metrics.iter().any(|m| m == metric)
    }

    fn build_record(&self, app: &App) -> TickRecord {
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        let cpu = if self.should_log("cpu") {
            Some(CpuSnapshot {
                total: app.cpu.total_usage(),
                cores: app.cpu.per_core_usage().to_vec(),
            })
        } else {
            None
        };

        let memory = if self.should_log("memory") {
            Some(MemorySnapshot {
                used: app.mem.used_ram(),
                total: app.mem.total_ram(),
                swap_used: app.mem.swap_used(),
                swap_total: app.mem.swap_total(),
            })
        } else {
            None
        };

        let disk = if self.should_log("disk") {
            Some(DiskSnapshot {
                read_bytes_sec: app.disk.total_read_bytes_sec(),
                write_bytes_sec: app.disk.total_write_bytes_sec(),
                filesystems: app
                    .disk
                    .disks()
                    .iter()
                    .map(|d| DiskEntry {
                        mount: d.mount_point.clone(),
                        total_bytes: d.total_bytes,
                        used_bytes: d.used_bytes,
                        free_bytes: d.free_bytes,
                        usage_percent: d.usage_percent,
                    })
                    .collect(),
            })
        } else {
            None
        };

        let network = if self.should_log("network") {
            Some(
                app.net
                    .interfaces()
                    .iter()
                    .map(|i| NetEntry {
                        name: i.name.clone(),
                        rx_bytes_sec: i.rx_bytes_sec,
                        tx_bytes_sec: i.tx_bytes_sec,
                        total_rx: i.total_rx,
                        total_tx: i.total_tx,
                    })
                    .collect(),
            )
        } else {
            None
        };

        TickRecord {
            timestamp,
            cpu,
            memory,
            disk,
            network,
        }
    }

    fn write_line(&mut self, line: &str) -> std::io::Result<()> {
        if let Some(ref mut w) = self.writer {
            w.write_all(line.as_bytes())?;
            w.flush()?;
            self.current_size += line.len() as u64;
        }
        Ok(())
    }

    fn should_rotate(&self) -> bool {
        match self.config.rotation.mode.as_str() {
            "size" => self.current_size >= self.config.rotation.max_size_bytes,
            "time" => self.rotation_start.elapsed().as_secs() >= self.config.rotation.max_age_secs,
            _ => false,
        }
    }

    fn rotate(&mut self) -> std::io::Result<()> {
        // Flush and drop the current writer
        self.writer.take();

        // Rename current file with a timestamp suffix
        let ts = Utc::now().format("%Y%m%dT%H%M%S");
        let ext = match self.config.format {
            LogFormat::Json => "jsonl",
            LogFormat::Csv => "csv",
        };
        let rotated = self.log_dir.join(format!("kite-metrics-{ts}.{ext}"));

        fs::rename(&self.current_file, &rotated)?;

        // Optionally gzip the rotated file
        if self.config.compress {
            if let Err(e) = compress_file(&rotated) {
                eprintln!(
                    "kite: gzip compression failed for {}: {e}",
                    rotated.display()
                );
            }
        }

        // Open a fresh log file
        self.writer = Some(open_append(&self.current_file)?);
        self.current_size = 0;
        self.rotation_start = Instant::now();
        self.csv_header_written = false;

        // Clean up old rotated files
        self.cleanup_old_files();

        Ok(())
    }

    fn cleanup_old_files(&self) {
        let prefix = "kite-metrics-";
        let mut rotated: Vec<PathBuf> = fs::read_dir(&self.log_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(prefix))
            })
            .collect();

        // Sort oldest first (lexicographic on timestamp in name)
        rotated.sort();

        let max = self.config.rotation.max_files;
        if rotated.len() > max {
            let to_remove = rotated.len() - max;
            for path in rotated.into_iter().take(to_remove) {
                let _ = fs::remove_file(&path);
            }
        }
    }

    // -- CSV formatting ---------------------------------------------------

    fn format_csv(&mut self, record: &TickRecord) -> String {
        let mut out = String::new();

        if !self.csv_header_written {
            out.push_str(&Self::csv_header(record));
            self.csv_header_written = true;
        }

        out.push_str(&Self::csv_row(record));
        out
    }

    fn csv_header(record: &TickRecord) -> String {
        let mut cols = vec!["timestamp".to_string()];

        if record.cpu.is_some() {
            cols.push("cpu_total".into());
        }
        if let Some(ref mem) = record.memory {
            let _ = mem;
            cols.extend([
                "mem_used".into(),
                "mem_total".into(),
                "swap_used".into(),
                "swap_total".into(),
            ]);
        }
        if record.disk.is_some() {
            cols.extend(["disk_read_bytes_sec".into(), "disk_write_bytes_sec".into()]);
        }
        if record.network.is_some() {
            cols.extend(["net_rx_bytes_sec".into(), "net_tx_bytes_sec".into()]);
        }

        cols.join(",") + "\n"
    }

    fn csv_row(record: &TickRecord) -> String {
        let mut vals: Vec<String> = vec![record.timestamp.clone()];

        if let Some(ref cpu) = record.cpu {
            vals.push(format!("{:.2}", cpu.total));
        }
        if let Some(ref mem) = record.memory {
            vals.push(mem.used.to_string());
            vals.push(mem.total.to_string());
            vals.push(mem.swap_used.to_string());
            vals.push(mem.swap_total.to_string());
        }
        if let Some(ref disk) = record.disk {
            vals.push(format!("{:.2}", disk.read_bytes_sec));
            vals.push(format!("{:.2}", disk.write_bytes_sec));
        }
        if let Some(ref net) = record.network {
            let rx: f64 = net.iter().map(|n| n.rx_bytes_sec).sum();
            let tx: f64 = net.iter().map(|n| n.tx_bytes_sec).sum();
            vals.push(format!("{:.2}", rx));
            vals.push(format!("{:.2}", tx));
        }

        vals.join(",") + "\n"
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn resolve_log_dir(config: &LoggingConfig) -> PathBuf {
    if let Some(ref p) = config.path {
        PathBuf::from(p)
    } else {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kite")
            .join("logs")
    }
}

fn open_append(path: &Path) -> std::io::Result<BufWriter<File>> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    Ok(BufWriter::new(file))
}

fn compress_file(path: &Path) -> std::io::Result<()> {
    let data = fs::read(path)?;
    let gz_path = PathBuf::from(format!("{}.gz", path.display()));
    let gz_file = File::create(&gz_path)?;
    let mut encoder = GzEncoder::new(gz_file, Compression::default());
    encoder.write_all(&data)?;
    encoder.finish()?;
    fs::remove_file(path)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::{LogFormat, LogRotation, LoggingConfig};
    use std::io::Read;

    fn test_config(dir: &Path, format: LogFormat) -> LoggingConfig {
        LoggingConfig {
            enabled: true,
            format,
            path: Some(dir.to_string_lossy().into_owned()),
            rotation: LogRotation {
                mode: "size".to_string(),
                max_size_bytes: 50 * 1024 * 1024,
                max_age_secs: 86400,
                max_files: 3,
            },
            compress: false,
            metrics: Vec::new(),
        }
    }

    #[test]
    fn disabled_logger_returns_none() {
        let config = LoggingConfig::default(); // enabled: false
        assert!(MetricsLogger::new(&config).is_none());
    }

    #[test]
    fn default_path_uses_data_dir() {
        let config = LoggingConfig {
            enabled: true,
            path: None,
            ..LoggingConfig::default()
        };
        let dir = resolve_log_dir(&config);
        let expected = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kite")
            .join("logs");
        assert_eq!(dir, expected);
    }

    #[test]
    fn custom_path_is_respected() {
        let config = LoggingConfig {
            enabled: true,
            path: Some("/my/custom/logs".to_string()),
            ..LoggingConfig::default()
        };
        let dir = resolve_log_dir(&config);
        assert_eq!(dir, PathBuf::from("/my/custom/logs"));
    }

    #[test]
    fn json_serialization_roundtrip() {
        let record = TickRecord {
            timestamp: "2026-04-18T12:00:00Z".to_string(),
            cpu: Some(CpuSnapshot {
                total: 45.2,
                cores: vec![30.1, 55.3],
            }),
            memory: Some(MemorySnapshot {
                used: 8_589_934_592,
                total: 17_179_869_184,
                swap_used: 0,
                swap_total: 4_294_967_296,
            }),
            disk: None,
            network: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"timestamp\":\"2026-04-18T12:00:00Z\""));
        assert!(json.contains("\"total\":45.2"));
        assert!(json.contains("\"cores\":[30.1,55.3]"));
        assert!(json.contains("\"used\":8589934592"));
        // disk and network should be omitted
        assert!(!json.contains("\"disk\""));
        assert!(!json.contains("\"network\""));
    }

    #[test]
    fn csv_format_produces_header_and_row() {
        let record = TickRecord {
            timestamp: "2026-04-18T12:00:00Z".to_string(),
            cpu: Some(CpuSnapshot {
                total: 50.0,
                cores: vec![],
            }),
            memory: Some(MemorySnapshot {
                used: 1024,
                total: 2048,
                swap_used: 0,
                swap_total: 512,
            }),
            disk: None,
            network: None,
        };

        let header = MetricsLogger::csv_header(&record);
        assert!(header.starts_with("timestamp,cpu_total,mem_used,mem_total,swap_used,swap_total"));

        let row = MetricsLogger::csv_row(&record);
        let parts: Vec<&str> = row.trim().split(',').collect();
        assert_eq!(parts[0], "2026-04-18T12:00:00Z");
        assert_eq!(parts[1], "50.00");
        assert_eq!(parts[2], "1024");
    }

    #[test]
    fn rotation_by_size_triggers() {
        let dir = std::env::current_dir().unwrap().join("test_rotation_size");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let mut config = test_config(&dir, LogFormat::Json);
        config.rotation.max_size_bytes = 100; // tiny threshold

        let mut logger = MetricsLogger::new(&config).unwrap();
        // Write enough to exceed threshold
        let big_line = "x".repeat(150) + "\n";
        logger.write_line(&big_line).unwrap();

        assert!(logger.should_rotate());

        // Perform rotation
        logger.rotate().unwrap();
        assert_eq!(logger.current_size, 0);

        // Rotated file exists
        let rotated: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("kite-metrics-"))
            .collect();
        assert!(!rotated.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cleanup_respects_max_files() {
        let dir = std::env::current_dir().unwrap().join("test_cleanup");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Create fake rotated files
        for i in 0..5 {
            let name = format!("kite-metrics-2026010{i}T120000.jsonl");
            File::create(dir.join(name)).unwrap();
        }

        let config = test_config(&dir, LogFormat::Json);
        let logger = MetricsLogger::new(&config).unwrap();
        logger.cleanup_old_files();

        let remaining: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("kite-metrics-"))
            .collect();
        // max_files is 3
        assert!(
            remaining.len() <= 3,
            "expected <= 3 files, got {}",
            remaining.len()
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn gzip_compression_works() {
        let dir = std::env::current_dir().unwrap().join("test_gzip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let src = dir.join("sample.jsonl");
        fs::write(&src, b"hello world\n").unwrap();

        compress_file(&src).unwrap();

        // Original removed
        assert!(!src.exists());

        // .gz file exists and is valid gzip
        let gz_path = dir.join("sample.jsonl.gz");
        assert!(gz_path.exists());

        let gz_data = fs::read(&gz_path).unwrap();
        let mut decoder = flate2::read::GzDecoder::new(&gz_data[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        assert_eq!(decompressed, "hello world\n");

        let _ = fs::remove_dir_all(&dir);
    }
}
