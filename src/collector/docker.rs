use anyhow::Result;

use crate::collector::Collector;

/// State of a Docker container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ContainerState {
    Running,
    Paused,
    Exited,
    Created,
    Restarting,
    Removing,
    Dead,
    Unknown,
}

impl std::fmt::Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerState::Running => write!(f, "Running"),
            ContainerState::Paused => write!(f, "Paused"),
            ContainerState::Exited => write!(f, "Exited"),
            ContainerState::Created => write!(f, "Created"),
            ContainerState::Restarting => write!(f, "Restarting"),
            ContainerState::Removing => write!(f, "Removing"),
            ContainerState::Dead => write!(f, "Dead"),
            ContainerState::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ContainerState {
    #[allow(dead_code)]
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" => ContainerState::Running,
            "paused" => ContainerState::Paused,
            "exited" => ContainerState::Exited,
            "created" => ContainerState::Created,
            "restarting" => ContainerState::Restarting,
            "removing" => ContainerState::Removing,
            "dead" => ContainerState::Dead,
            _ => ContainerState::Unknown,
        }
    }
}

/// Information about a single Docker container.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: ContainerState,
    pub cpu_percent: f64,
    pub memory_used: u64,
    pub memory_limit: u64,
    pub net_rx: u64,
    pub net_tx: u64,
    pub block_read: u64,
    pub block_write: u64,
}

/// Collects Docker container metrics via bollard (when the `docker` feature is enabled).
pub struct DockerCollector {
    #[cfg(feature = "docker")]
    client: Option<bollard::Docker>,
    containers: Vec<ContainerInfo>,
}

// ── Feature-gated implementation: docker ON ─────────────────────────────────

#[cfg(feature = "docker")]
#[allow(dead_code)]
impl DockerCollector {
    pub fn new() -> Self {
        let client = bollard::Docker::connect_with_local_defaults().ok();
        Self {
            client,
            containers: Vec::new(),
        }
    }

    async fn collect_async(&mut self) -> Result<()> {
        use bollard::container::{ListContainersOptions, StatsOptions};
        use futures::StreamExt;

        let client = match &self.client {
            Some(c) => c,
            None => {
                self.containers.clear();
                return Ok(());
            }
        };

        let opts = ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        };

        let summaries = match client.list_containers(Some(opts)).await {
            Ok(s) => s,
            Err(_) => {
                self.containers.clear();
                return Ok(());
            }
        };

        let mut new_containers = Vec::with_capacity(summaries.len());

        for summary in &summaries {
            let id_full = summary.id.as_deref().unwrap_or_default();
            let id = if id_full.len() > 12 {
                id_full[..12].to_string()
            } else {
                id_full.to_string()
            };

            let name = summary
                .names
                .as_ref()
                .and_then(|names| names.first())
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default();

            let image = summary.image.clone().unwrap_or_default();
            let status = summary.status.clone().unwrap_or_default();
            let state = ContainerState::from_str(summary.state.as_deref().unwrap_or("unknown"));

            let mut cpu_percent = 0.0;
            let mut memory_used = 0u64;
            let mut memory_limit = 0u64;
            let mut net_rx = 0u64;
            let mut net_tx = 0u64;
            let mut block_read = 0u64;
            let mut block_write = 0u64;

            // Only fetch stats for running containers
            if state == ContainerState::Running {
                let stats_opts = StatsOptions {
                    stream: false,
                    one_shot: true,
                };
                let mut stream = client.stats(id_full, Some(stats_opts));
                if let Some(Ok(stats)) = stream.next().await {
                    // CPU calculation
                    let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
                        - stats.precpu_stats.cpu_usage.total_usage as f64;
                    let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
                        - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
                    let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

                    if system_delta > 0.0 {
                        cpu_percent = (cpu_delta / system_delta) * num_cpus * 100.0;
                    }

                    // Memory
                    memory_used = stats.memory_stats.usage.unwrap_or(0);
                    memory_limit = stats.memory_stats.limit.unwrap_or(0);

                    // Network I/O
                    if let Some(networks) = &stats.networks {
                        for iface in networks.values() {
                            net_rx += iface.rx_bytes;
                            net_tx += iface.tx_bytes;
                        }
                    }

                    // Block I/O
                    if let Some(ref io_service) = stats.blkio_stats.io_service_bytes_recursive {
                        for entry in io_service {
                            match entry.op.to_lowercase().as_str() {
                                "read" => block_read += entry.value,
                                "write" => block_write += entry.value,
                                _ => {}
                            }
                        }
                    }
                }
            }

            new_containers.push(ContainerInfo {
                id,
                name,
                image,
                status,
                state,
                cpu_percent,
                memory_used,
                memory_limit,
                net_rx,
                net_tx,
                block_read,
                block_write,
            });
        }

        self.containers = new_containers;
        Ok(())
    }

    /// Start a stopped container.
    pub fn start_container(&self, id: &str) -> Result<()> {
        use bollard::container::StartContainerOptions;

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Docker client not available"))?;
        let id = id.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                client
                    .start_container(&id, None::<StartContainerOptions<String>>)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to start container: {}", e))
            })
        })
    }

    /// Stop a running container.
    pub fn stop_container(&self, id: &str) -> Result<()> {
        use bollard::container::StopContainerOptions;

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Docker client not available"))?;
        let id = id.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                client
                    .stop_container(&id, None::<StopContainerOptions>)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to stop container: {}", e))
            })
        })
    }

    /// Restart a container.
    pub fn restart_container(&self, id: &str) -> Result<()> {
        use bollard::container::RestartContainerOptions;

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Docker client not available"))?;
        let id = id.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                client
                    .restart_container(&id, None::<RestartContainerOptions>)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to restart container: {}", e))
            })
        })
    }
}

#[cfg(feature = "docker")]
impl Collector for DockerCollector {
    fn collect(&mut self) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.collect_async())
        })
    }
}

// ── Feature-gated implementation: docker OFF ────────────────────────────────

#[cfg(not(feature = "docker"))]
#[allow(dead_code)]
impl DockerCollector {
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
        }
    }

    pub fn start_container(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    pub fn stop_container(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    pub fn restart_container(&self, _id: &str) -> Result<()> {
        Ok(())
    }
}

#[cfg(not(feature = "docker"))]
impl Collector for DockerCollector {
    fn collect(&mut self) -> Result<()> {
        Ok(())
    }
}

// ── Shared getters (always available) ───────────────────────────────────────

impl DockerCollector {
    pub fn containers(&self) -> &[ContainerInfo] {
        &self.containers
    }

    pub fn has_docker(&self) -> bool {
        #[cfg(feature = "docker")]
        {
            self.client.is_some() || !self.containers.is_empty()
        }
        #[cfg(not(feature = "docker"))]
        {
            false
        }
    }

    pub fn container_count(&self) -> usize {
        self.containers.len()
    }

    pub fn running_count(&self) -> usize {
        self.containers
            .iter()
            .filter(|c| c.state == ContainerState::Running)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes() {
        let collector = DockerCollector::new();
        assert_eq!(collector.container_count(), 0);
        assert!(collector.containers().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn collect_succeeds() {
        // Docker may not be available in CI — just ensure no panic
        let mut collector = DockerCollector::new();
        let _ = collector.collect();
    }

    #[test]
    fn has_docker_consistent() {
        let collector = DockerCollector::new();
        // Without collecting, containers is empty, so has_docker depends on client
        let _ = collector.has_docker();
    }

    #[test]
    fn stub_works() {
        let collector = DockerCollector::new();
        assert_eq!(collector.container_count(), 0);
        assert_eq!(collector.running_count(), 0);
        assert!(collector.containers().is_empty());
    }

    #[test]
    fn container_state_display() {
        assert_eq!(ContainerState::Running.to_string(), "Running");
        assert_eq!(ContainerState::Paused.to_string(), "Paused");
        assert_eq!(ContainerState::Exited.to_string(), "Exited");
        assert_eq!(ContainerState::Created.to_string(), "Created");
        assert_eq!(ContainerState::Restarting.to_string(), "Restarting");
        assert_eq!(ContainerState::Removing.to_string(), "Removing");
        assert_eq!(ContainerState::Dead.to_string(), "Dead");
        assert_eq!(ContainerState::Unknown.to_string(), "Unknown");

        // Test from_str
        assert_eq!(ContainerState::from_str("running"), ContainerState::Running);
        assert_eq!(ContainerState::from_str("PAUSED"), ContainerState::Paused);
        assert_eq!(ContainerState::from_str("exited"), ContainerState::Exited);
        assert_eq!(
            ContainerState::from_str("anything"),
            ContainerState::Unknown
        );
    }
}
