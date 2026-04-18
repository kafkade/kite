use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use sysinfo::{Pid, ProcessesToUpdate, System};

use super::Collector;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu_percent: f32,
    pub mem_percent: f32,
    pub mem_bytes: u64,
    pub status: String,
    pub parent_pid: Option<u32>,
    pub command: String,
    pub threads: Option<u32>,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Pid,
    Name,
    User,
    Cpu,
    Memory,
    Status,
    Threads,
}

const SORT_COLUMNS: [SortColumn; 7] = [
    SortColumn::Pid,
    SortColumn::Name,
    SortColumn::User,
    SortColumn::Cpu,
    SortColumn::Memory,
    SortColumn::Status,
    SortColumn::Threads,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TreeNode {
    pub process: ProcessInfo,
    pub depth: usize,
    pub children_count: usize,
}

pub struct ProcessCollector {
    system: System,
    all_processes: Vec<ProcessInfo>,
    filtered: Vec<ProcessInfo>,
    sort_column: SortColumn,
    sort_order: SortOrder,
    filter: Option<String>,
    paused: bool,
    top_n: Option<usize>,
    bookmarks: HashSet<u32>,
}

#[allow(dead_code)]
impl ProcessCollector {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            all_processes: Vec::new(),
            filtered: Vec::new(),
            sort_column: SortColumn::Cpu,
            sort_order: SortOrder::Descending,
            filter: None,
            paused: false,
            top_n: None,
            bookmarks: HashSet::new(),
        }
    }

    pub fn processes(&self) -> &[ProcessInfo] {
        &self.filtered
    }

    pub fn all_process_count(&self) -> usize {
        self.all_processes.len()
    }

    pub fn set_sort(&mut self, column: SortColumn, order: SortOrder) {
        self.sort_column = column;
        self.sort_order = order;
        self.apply_filter_and_sort();
    }

    pub fn sort_column(&self) -> SortColumn {
        self.sort_column
    }

    pub fn sort_order(&self) -> SortOrder {
        self.sort_order
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
        self.apply_filter_and_sort();
    }

    pub fn next_sort_column(&mut self) {
        let idx = SORT_COLUMNS
            .iter()
            .position(|&c| c == self.sort_column)
            .unwrap_or(0);
        self.sort_column = SORT_COLUMNS[(idx + 1) % SORT_COLUMNS.len()];
        self.apply_filter_and_sort();
    }

    pub fn prev_sort_column(&mut self) {
        let idx = SORT_COLUMNS
            .iter()
            .position(|&c| c == self.sort_column)
            .unwrap_or(0);
        self.sort_column = if idx == 0 {
            SORT_COLUMNS[SORT_COLUMNS.len() - 1]
        } else {
            SORT_COLUMNS[idx - 1]
        };
        self.apply_filter_and_sort();
    }

    pub fn set_filter(&mut self, filter: Option<String>) {
        self.filter = filter;
        self.apply_filter_and_sort();
    }

    pub fn filter(&self) -> Option<&str> {
        self.filter.as_deref()
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_top_n(&mut self, n: Option<usize>) {
        self.top_n = n;
        self.apply_filter_and_sort();
    }

    pub fn top_n(&self) -> Option<usize> {
        self.top_n
    }

    pub fn cycle_top_n(&mut self) {
        self.top_n = match self.top_n {
            None => Some(10),
            Some(10) => Some(25),
            Some(25) => Some(50),
            _ => None,
        };
        self.apply_filter_and_sort();
    }

    pub fn toggle_bookmark(&mut self, pid: u32) {
        if self.bookmarks.contains(&pid) {
            self.bookmarks.remove(&pid);
        } else {
            self.bookmarks.insert(pid);
        }
        self.apply_filter_and_sort();
    }

    pub fn is_bookmarked(&self, pid: u32) -> bool {
        self.bookmarks.contains(&pid)
    }

    pub fn bookmarks(&self) -> &HashSet<u32> {
        &self.bookmarks
    }

    pub fn clear_bookmarks(&mut self) {
        self.bookmarks.clear();
        self.apply_filter_and_sort();
    }

    pub fn kill_process(&self, pid: u32) -> Result<()> {
        let sysinfo_pid = Pid::from_u32(pid);
        let process = self
            .system
            .process(sysinfo_pid)
            .context(format!("Process with PID {pid} not found"))?;
        process.kill();
        Ok(())
    }

    pub fn tree(&self) -> Vec<TreeNode> {
        // Build parent -> children map from the filtered list
        let mut children_map: HashMap<Option<u32>, Vec<&ProcessInfo>> = HashMap::new();
        for p in &self.filtered {
            children_map.entry(p.parent_pid).or_default().push(p);
        }

        // Find the set of PIDs present in filtered
        let pid_set: std::collections::HashSet<u32> = self.filtered.iter().map(|p| p.pid).collect();

        // Roots: processes whose parent is None or whose parent is not in the filtered set
        let mut roots: Vec<&ProcessInfo> = self
            .filtered
            .iter()
            .filter(|p| match p.parent_pid {
                None => true,
                Some(ppid) => !pid_set.contains(&ppid),
            })
            .collect();
        roots.sort_by_key(|a| a.pid);

        let mut result = Vec::new();
        for root in roots {
            Self::build_tree_dfs(root, 0, &children_map, &mut result);
        }
        result
    }

    fn build_tree_dfs(
        proc: &ProcessInfo,
        depth: usize,
        children_map: &HashMap<Option<u32>, Vec<&ProcessInfo>>,
        result: &mut Vec<TreeNode>,
    ) {
        let children = children_map.get(&Some(proc.pid));
        let children_count = children.map_or(0, |c| c.len());
        result.push(TreeNode {
            process: proc.clone(),
            depth,
            children_count,
        });
        if let Some(kids) = children {
            let mut sorted_kids = kids.clone();
            sorted_kids.sort_by_key(|a| a.pid);
            for child in sorted_kids {
                Self::build_tree_dfs(child, depth + 1, children_map, result);
            }
        }
    }

    fn apply_filter_and_sort(&mut self) {
        let mut list = if let Some(ref filter) = self.filter {
            let lower = filter.to_lowercase();
            self.all_processes
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&lower)
                        || p.command.to_lowercase().contains(&lower)
                })
                .cloned()
                .collect()
        } else {
            self.all_processes.clone()
        };

        let order = self.sort_order;
        list.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::User => a.user.cmp(&b.user),
                SortColumn::Cpu => a
                    .cpu_percent
                    .partial_cmp(&b.cpu_percent)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Memory => a.mem_bytes.cmp(&b.mem_bytes),
                SortColumn::Status => a.status.cmp(&b.status),
                SortColumn::Threads => a.threads.cmp(&b.threads),
            };
            match order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });

        self.filtered = list;

        // Move bookmarked processes to the top (both groups keep their sort order)
        if !self.bookmarks.is_empty() {
            let mut bookmarked: Vec<ProcessInfo> = Vec::new();
            let mut unbookmarked: Vec<ProcessInfo> = Vec::new();
            for p in self.filtered.drain(..) {
                if self.bookmarks.contains(&p.pid) {
                    bookmarked.push(p);
                } else {
                    unbookmarked.push(p);
                }
            }
            self.filtered = bookmarked;
            self.filtered.extend(unbookmarked);
        }

        // Truncate to top_n if set
        if let Some(n) = self.top_n {
            self.filtered.truncate(n);
        }
    }
}

impl Collector for ProcessCollector {
    fn collect(&mut self) -> Result<()> {
        if self.paused {
            return Ok(());
        }

        self.system.refresh_processes(ProcessesToUpdate::All, true);

        let total_memory = self.system.total_memory();
        let mut processes = Vec::new();

        for process in self.system.processes().values() {
            let pid = process.pid().as_u32();
            let name = process.name().to_string_lossy().to_string();
            let cpu_percent = process.cpu_usage();
            let mem_bytes = process.memory();
            let mem_percent = if total_memory > 0 {
                (mem_bytes as f64 / total_memory as f64 * 100.0) as f32
            } else {
                0.0
            };
            let status = format!("{:?}", process.status());
            let parent_pid = process.parent().map(|p| p.as_u32());
            let command = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let threads = process.tasks().map(|t| t.len() as u32);

            let disk_usage = process.disk_usage();

            let user = match process.user_id() {
                Some(uid) => format!("{uid:?}"),
                None => "N/A".to_string(),
            };

            processes.push(ProcessInfo {
                pid,
                name,
                user,
                cpu_percent,
                mem_percent,
                mem_bytes,
                status,
                parent_pid,
                command,
                threads,
                disk_read_bytes: disk_usage.total_read_bytes,
                disk_write_bytes: disk_usage.total_written_bytes,
            });
        }

        self.all_processes = processes;
        self.apply_filter_and_sort();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_collect() {
        let mut collector = ProcessCollector::new();
        assert!(collector.collect().is_ok());
    }

    #[test]
    fn test_has_processes_after_collect() {
        let mut collector = ProcessCollector::new();
        collector.collect().unwrap();
        assert!(collector.all_process_count() >= 1);
        assert!(!collector.processes().is_empty());
    }

    #[test]
    fn test_sort_switching() {
        let mut collector = ProcessCollector::new();
        collector.collect().unwrap();

        assert_eq!(collector.sort_column(), SortColumn::Cpu);
        assert_eq!(collector.sort_order(), SortOrder::Descending);

        collector.next_sort_column();
        assert_eq!(collector.sort_column(), SortColumn::Memory);

        collector.prev_sort_column();
        assert_eq!(collector.sort_column(), SortColumn::Cpu);

        collector.toggle_sort_order();
        assert_eq!(collector.sort_order(), SortOrder::Ascending);

        collector.set_sort(SortColumn::Pid, SortOrder::Ascending);
        assert_eq!(collector.sort_column(), SortColumn::Pid);

        // Verify PID ascending order
        let procs = collector.processes();
        if procs.len() >= 2 {
            assert!(procs[0].pid <= procs[1].pid);
        }
    }

    #[test]
    fn test_filter_applies() {
        let mut collector = ProcessCollector::new();
        collector.collect().unwrap();

        let total = collector.all_process_count();

        // Filter with a very specific string that likely matches few/no processes
        collector.set_filter(Some("zzz_nonexistent_proc_zzz".to_string()));
        assert!(collector.processes().len() <= total);

        // Reset filter
        collector.set_filter(None);
        assert_eq!(collector.processes().len(), total);
    }

    #[test]
    fn test_pause_prevents_update() {
        let mut collector = ProcessCollector::new();
        collector.collect().unwrap();
        let count_before = collector.all_process_count();

        collector.toggle_pause();
        assert!(collector.is_paused());

        // Collect while paused should not change data
        collector.collect().unwrap();
        assert_eq!(collector.all_process_count(), count_before);

        collector.toggle_pause();
        assert!(!collector.is_paused());
    }

    #[test]
    fn test_tree_non_empty() {
        let mut collector = ProcessCollector::new();
        collector.collect().unwrap();
        let tree = collector.tree();
        assert!(!tree.is_empty());
        // Root nodes should have depth 0
        assert_eq!(tree[0].depth, 0);
    }

    #[test]
    fn test_top_n_truncates() {
        let mut collector = ProcessCollector::new();
        collector.collect().unwrap();
        collector.set_top_n(Some(5));
        assert!(collector.processes().len() <= 5);
    }

    #[test]
    fn test_top_n_cycle() {
        let mut collector = ProcessCollector::new();
        assert_eq!(collector.top_n(), None);
        collector.cycle_top_n();
        assert_eq!(collector.top_n(), Some(10));
        collector.cycle_top_n();
        assert_eq!(collector.top_n(), Some(25));
        collector.cycle_top_n();
        assert_eq!(collector.top_n(), Some(50));
        collector.cycle_top_n();
        assert_eq!(collector.top_n(), None);
    }

    #[test]
    fn test_bookmark_toggle() {
        let mut collector = ProcessCollector::new();
        assert!(!collector.is_bookmarked(1234));
        collector.toggle_bookmark(1234);
        assert!(collector.is_bookmarked(1234));
        collector.toggle_bookmark(1234);
        assert!(!collector.is_bookmarked(1234));
    }

    #[test]
    fn test_bookmarked_processes_first() {
        let mut collector = ProcessCollector::new();
        collector.collect().unwrap();

        let procs = collector.processes();
        if procs.len() < 3 {
            return; // not enough processes to test ordering
        }

        // Bookmark the last process in the current list
        let last_pid = procs[procs.len() - 1].pid;
        collector.toggle_bookmark(last_pid);

        // After re-sorting, the bookmarked process should be first
        let procs = collector.processes();
        assert_eq!(procs[0].pid, last_pid);
    }

    #[test]
    fn test_io_fields_present() {
        let mut collector = ProcessCollector::new();
        collector.collect().unwrap();
        // I/O fields should exist on every process (values may be 0)
        for p in collector.processes() {
            let _ = p.disk_read_bytes;
            let _ = p.disk_write_bytes;
        }
    }
}
