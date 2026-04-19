//! Data replay engine for post-mortem analysis.
//!
//! Loads a JSON Lines metrics log file (produced by the logging module) and
//! allows seeking through recorded snapshots in the TUI.

use std::fs;
use std::path::Path;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Deserialization types (mirror export::log serialization types)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ReplayRecord {
    pub timestamp: String,
    pub cpu: Option<CpuSnapshot>,
    pub memory: Option<MemorySnapshot>,
    pub disk: Option<DiskSnapshot>,
    pub network: Option<Vec<NetEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CpuSnapshot {
    pub total: f64,
    pub cores: Vec<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemorySnapshot {
    pub used: u64,
    pub total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiskSnapshot {
    pub read_bytes_sec: f64,
    pub write_bytes_sec: f64,
    pub filesystems: Vec<DiskEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiskEntry {
    pub mount: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetEntry {
    pub name: String,
    pub rx_bytes_sec: f64,
    pub tx_bytes_sec: f64,
    pub total_rx: u64,
    pub total_tx: u64,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("failed to read replay file: {0}")]
    Io(#[from] std::io::Error),

    #[error("no valid records found in replay file")]
    EmptyFile,
}

// ---------------------------------------------------------------------------
// ReplayState
// ---------------------------------------------------------------------------

/// Holds a loaded replay session and the current playback position.
pub struct ReplayState {
    records: Vec<ReplayRecord>,
    position: usize,
    pub auto_play: bool,
}

impl ReplayState {
    /// Load a JSON Lines replay file. Skips lines that fail to parse.
    pub fn load(path: &str) -> Result<Self, ReplayError> {
        let content = fs::read_to_string(Path::new(path))?;
        let mut records = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(record) = serde_json::from_str::<ReplayRecord>(trimmed) {
                records.push(record);
            }
        }

        if records.is_empty() {
            return Err(ReplayError::EmptyFile);
        }

        Ok(Self {
            records,
            position: 0,
            auto_play: false,
        })
    }

    /// Returns the current record.
    pub fn current(&self) -> &ReplayRecord {
        &self.records[self.position]
    }

    /// Advance to the next record. Returns `true` if position changed.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if self.position + 1 < self.records.len() {
            self.position += 1;
            true
        } else {
            self.auto_play = false;
            false
        }
    }

    /// Go back to the previous record. Returns `true` if position changed.
    pub fn prev(&mut self) -> bool {
        if self.position > 0 {
            self.position -= 1;
            true
        } else {
            false
        }
    }

    /// Jump to the first record.
    pub fn seek_start(&mut self) {
        self.position = 0;
    }

    /// Jump to the last record.
    pub fn seek_end(&mut self) {
        self.position = self.records.len().saturating_sub(1);
    }

    /// Current position (0-indexed).
    pub fn position(&self) -> usize {
        self.position
    }

    /// Total number of records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the replay file has no records.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Timestamp of the current record.
    pub fn timestamp(&self) -> &str {
        &self.records[self.position].timestamp
    }

    /// Toggle auto-play on/off.
    pub fn toggle_auto_play(&mut self) {
        self.auto_play = !self.auto_play;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_test_file(dir: &Path, name: &str, content: &str) -> String {
        let path = dir.join(name);
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path.to_string_lossy().into_owned()
    }

    fn sample_jsonl() -> &'static str {
        concat!(
            r#"{"timestamp":"2026-04-18T12:00:00Z","cpu":{"total":45.2,"cores":[30.1,55.3]},"memory":{"used":8589934592,"total":17179869184,"swap_used":0,"swap_total":4294967296}}"#,
            "\n",
            r#"{"timestamp":"2026-04-18T12:00:01Z","cpu":{"total":50.0,"cores":[40.0,60.0]},"memory":{"used":9000000000,"total":17179869184,"swap_used":100000,"swap_total":4294967296}}"#,
            "\n",
            r#"{"timestamp":"2026-04-18T12:00:02Z","cpu":{"total":30.5,"cores":[20.0,41.0]}}"#,
        )
    }

    #[test]
    fn load_valid_file() {
        let dir = std::env::current_dir().unwrap().join("test_replay_load");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = write_test_file(&dir, "test.jsonl", sample_jsonl());
        let state = ReplayState::load(&path).unwrap();

        assert_eq!(state.len(), 3);
        assert_eq!(state.position(), 0);
        assert_eq!(state.timestamp(), "2026-04-18T12:00:00Z");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_empty_file_errors() {
        let dir = std::env::current_dir().unwrap().join("test_replay_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = write_test_file(&dir, "empty.jsonl", "");
        let result = ReplayState::load(&path);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_skips_invalid_lines() {
        let dir = std::env::current_dir().unwrap().join("test_replay_invalid");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let content = format!(
            "not valid json\n{}\nmore garbage",
            sample_jsonl().lines().next().unwrap()
        );
        let path = write_test_file(&dir, "mixed.jsonl", &content);
        let state = ReplayState::load(&path).unwrap();

        assert_eq!(state.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seek_forward_backward() {
        let dir = std::env::current_dir().unwrap().join("test_replay_seek");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = write_test_file(&dir, "test.jsonl", sample_jsonl());
        let mut state = ReplayState::load(&path).unwrap();

        assert_eq!(state.position(), 0);
        assert!(state.next());
        assert_eq!(state.position(), 1);
        assert!(state.next());
        assert_eq!(state.position(), 2);
        assert!(!state.next());
        assert_eq!(state.position(), 2);

        assert!(state.prev());
        assert_eq!(state.position(), 1);
        assert!(state.prev());
        assert_eq!(state.position(), 0);
        assert!(!state.prev());
        assert_eq!(state.position(), 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seek_start_end() {
        let dir = std::env::current_dir()
            .unwrap()
            .join("test_replay_seek_edges");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = write_test_file(&dir, "test.jsonl", sample_jsonl());
        let mut state = ReplayState::load(&path).unwrap();

        state.seek_end();
        assert_eq!(state.position(), 2);

        state.seek_start();
        assert_eq!(state.position(), 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn auto_play_stops_at_end() {
        let dir = std::env::current_dir()
            .unwrap()
            .join("test_replay_autoplay");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = write_test_file(&dir, "test.jsonl", sample_jsonl());
        let mut state = ReplayState::load(&path).unwrap();

        state.auto_play = true;
        state.seek_end();
        assert!(!state.next());
        assert!(!state.auto_play);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn current_returns_correct_record() {
        let dir = std::env::current_dir().unwrap().join("test_replay_current");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = write_test_file(&dir, "test.jsonl", sample_jsonl());
        let mut state = ReplayState::load(&path).unwrap();

        let record = state.current();
        assert!(record.cpu.is_some());
        assert!((record.cpu.as_ref().unwrap().total - 45.2).abs() < f64::EPSILON);

        state.next();
        let record = state.current();
        assert!((record.cpu.as_ref().unwrap().total - 50.0).abs() < f64::EPSILON);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn toggle_auto_play() {
        let dir = std::env::current_dir().unwrap().join("test_replay_toggle");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let path = write_test_file(&dir, "test.jsonl", sample_jsonl());
        let mut state = ReplayState::load(&path).unwrap();

        assert!(!state.auto_play);
        state.toggle_auto_play();
        assert!(state.auto_play);
        state.toggle_auto_play();
        assert!(!state.auto_play);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn nonexistent_file_returns_io_error() {
        let result = ReplayState::load("nonexistent_replay_file_path.jsonl");
        assert!(result.is_err());
    }
}
