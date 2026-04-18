use std::time::Duration;

use anyhow::Result;

use crate::collector::Collector;

/// Information about a single Kubernetes pod.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: PodStatus,
    pub ready: String,
    pub restarts: u32,
    pub age: Duration,
    pub node: String,
    pub cpu_request: Option<String>,
    pub cpu_limit: Option<String>,
    pub mem_request: Option<String>,
    pub mem_limit: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PodStatus {
    Running,
    Pending,
    Succeeded,
    Failed,
    Unknown,
    CrashLoopBackOff,
    ContainerCreating,
    Terminating,
    ImagePullBackOff,
}

impl std::fmt::Display for PodStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PodStatus::Running => write!(f, "Running"),
            PodStatus::Pending => write!(f, "Pending"),
            PodStatus::Succeeded => write!(f, "Succeeded"),
            PodStatus::Failed => write!(f, "Failed"),
            PodStatus::Unknown => write!(f, "Unknown"),
            PodStatus::CrashLoopBackOff => write!(f, "CrashLoop"),
            PodStatus::ContainerCreating => write!(f, "Creating"),
            PodStatus::Terminating => write!(f, "Terminating"),
            PodStatus::ImagePullBackOff => write!(f, "ImgPull"),
        }
    }
}

/// Collects Kubernetes pod metrics via kube-rs (when the `k8s` feature is enabled).
pub struct K8sCollector {
    #[cfg(feature = "k8s")]
    client: Option<kube::Client>,
    pods: Vec<PodInfo>,
    namespace_filter: Option<String>,
    available_namespaces: Vec<String>,
}

// ── Feature-gated implementation: k8s ON ────────────────────────────────────

#[cfg(feature = "k8s")]
#[allow(dead_code)]
impl K8sCollector {
    pub fn new() -> Self {
        let client = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { kube::Client::try_default().await.ok() })
        });
        Self {
            client,
            pods: Vec::new(),
            namespace_filter: None,
            available_namespaces: Vec::new(),
        }
    }

    async fn collect_async(&mut self) -> Result<()> {
        use k8s_openapi::api::core::v1::Pod;
        use kube::Api;
        use kube::api::ListParams;

        let client = match &self.client {
            Some(c) => c.clone(),
            None => {
                self.pods.clear();
                return Ok(());
            }
        };

        let pods_api: Api<Pod> = match &self.namespace_filter {
            Some(ns) => Api::namespaced(client, ns),
            None => Api::all(client),
        };

        let pod_list = match pods_api.list(&ListParams::default()).await {
            Ok(list) => list,
            Err(_) => {
                self.pods.clear();
                return Ok(());
            }
        };

        let mut new_pods = Vec::with_capacity(pod_list.items.len());
        let mut namespaces = std::collections::BTreeSet::new();

        let now = chrono::Utc::now();

        for pod in &pod_list.items {
            let name = pod.metadata.name.clone().unwrap_or_default();
            let namespace = pod.metadata.namespace.clone().unwrap_or_default();
            namespaces.insert(namespace.clone());

            let status_obj = pod.status.as_ref();
            let phase = status_obj.and_then(|s| s.phase.as_deref());
            let container_statuses = status_obj.and_then(|s| s.container_statuses.as_ref());
            let has_deletion = pod.metadata.deletion_timestamp.is_some();

            // Determine pod status
            let pod_status = if has_deletion {
                PodStatus::Terminating
            } else if let Some(cs) = container_statuses {
                // Check waiting reasons first
                let waiting_reason = cs.iter().find_map(|c| {
                    c.state
                        .as_ref()
                        .and_then(|s| s.waiting.as_ref().and_then(|w| w.reason.as_deref()))
                });
                match waiting_reason {
                    Some("CrashLoopBackOff") => PodStatus::CrashLoopBackOff,
                    Some("ImagePullBackOff") => PodStatus::ImagePullBackOff,
                    Some("ContainerCreating") => PodStatus::ContainerCreating,
                    _ => Self::phase_to_status(phase),
                }
            } else {
                Self::phase_to_status(phase)
            };

            // Ready count
            let (ready_count, total_count) = container_statuses
                .map(|cs| {
                    let total = cs.len();
                    let ready = cs.iter().filter(|c| c.ready).count();
                    (ready, total)
                })
                .unwrap_or((0, 0));
            let ready = format!("{}/{}", ready_count, total_count);

            // Total restarts
            let restarts = container_statuses
                .map(|cs| cs.iter().map(|c| c.restart_count as u32).sum())
                .unwrap_or(0);

            // Age
            let age = pod
                .metadata
                .creation_timestamp
                .as_ref()
                .map(|t| {
                    let created = t.0;
                    let diff = now - created;
                    Duration::from_secs(diff.num_seconds().max(0) as u64)
                })
                .unwrap_or_default();

            // Node
            let node = pod
                .spec
                .as_ref()
                .and_then(|s| s.node_name.clone())
                .unwrap_or_default();

            // Resources from first container
            let (cpu_request, cpu_limit, mem_request, mem_limit) = pod
                .spec
                .as_ref()
                .and_then(|s| s.containers.first())
                .and_then(|c| c.resources.as_ref())
                .map(|res| {
                    let cpu_req = res
                        .requests
                        .as_ref()
                        .and_then(|r| r.get("cpu"))
                        .map(|q| q.0.clone());
                    let cpu_lim = res
                        .limits
                        .as_ref()
                        .and_then(|r| r.get("cpu"))
                        .map(|q| q.0.clone());
                    let mem_req = res
                        .requests
                        .as_ref()
                        .and_then(|r| r.get("memory"))
                        .map(|q| q.0.clone());
                    let mem_lim = res
                        .limits
                        .as_ref()
                        .and_then(|r| r.get("memory"))
                        .map(|q| q.0.clone());
                    (cpu_req, cpu_lim, mem_req, mem_lim)
                })
                .unwrap_or((None, None, None, None));

            new_pods.push(PodInfo {
                name,
                namespace,
                status: pod_status,
                ready,
                restarts,
                age,
                node,
                cpu_request,
                cpu_limit,
                mem_request,
                mem_limit,
            });
        }

        self.pods = new_pods;
        self.available_namespaces = namespaces.into_iter().collect();
        Ok(())
    }

    fn phase_to_status(phase: Option<&str>) -> PodStatus {
        match phase {
            Some("Running") => PodStatus::Running,
            Some("Pending") => PodStatus::Pending,
            Some("Succeeded") => PodStatus::Succeeded,
            Some("Failed") => PodStatus::Failed,
            _ => PodStatus::Unknown,
        }
    }
}

#[cfg(feature = "k8s")]
impl Collector for K8sCollector {
    fn collect(&mut self) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.collect_async())
        })
    }
}

// ── Feature-gated implementation: k8s OFF ───────────────────────────────────

#[cfg(not(feature = "k8s"))]
#[allow(dead_code)]
impl K8sCollector {
    pub fn new() -> Self {
        Self {
            pods: Vec::new(),
            namespace_filter: None,
            available_namespaces: Vec::new(),
        }
    }
}

#[cfg(not(feature = "k8s"))]
impl Collector for K8sCollector {
    fn collect(&mut self) -> Result<()> {
        Ok(())
    }
}

// ── Shared getters (always available) ───────────────────────────────────────

#[allow(dead_code)]
impl K8sCollector {
    pub fn pods(&self) -> &[PodInfo] {
        &self.pods
    }

    pub fn has_k8s(&self) -> bool {
        #[cfg(feature = "k8s")]
        {
            self.client.is_some() || !self.pods.is_empty()
        }
        #[cfg(not(feature = "k8s"))]
        {
            false
        }
    }

    pub fn pod_count(&self) -> usize {
        self.pods.len()
    }

    pub fn running_count(&self) -> usize {
        self.pods
            .iter()
            .filter(|p| p.status == PodStatus::Running)
            .count()
    }

    pub fn set_namespace_filter(&mut self, ns: Option<String>) {
        self.namespace_filter = ns;
    }

    pub fn namespace_filter(&self) -> Option<&str> {
        self.namespace_filter.as_deref()
    }

    pub fn available_namespaces(&self) -> &[String] {
        &self.available_namespaces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes() {
        // Without a tokio runtime the stub path still works
        #[cfg(not(feature = "k8s"))]
        {
            let collector = K8sCollector::new();
            assert_eq!(collector.pod_count(), 0);
            assert!(collector.pods().is_empty());
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn new_initializes_async() {
        let collector = K8sCollector::new();
        assert_eq!(collector.pod_count(), 0);
        assert!(collector.pods().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn collect_succeeds() {
        // k8s cluster may not be available — just ensure no panic
        let mut collector = K8sCollector::new();
        let _ = collector.collect();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn has_k8s_consistent() {
        let collector = K8sCollector::new();
        let _ = collector.has_k8s();
    }

    #[test]
    fn stub_works() {
        #[cfg(not(feature = "k8s"))]
        {
            let collector = K8sCollector::new();
            assert_eq!(collector.pod_count(), 0);
            assert_eq!(collector.running_count(), 0);
            assert!(collector.pods().is_empty());
            assert!(!collector.has_k8s());
        }
    }

    #[test]
    fn pod_status_display() {
        assert_eq!(PodStatus::Running.to_string(), "Running");
        assert_eq!(PodStatus::Pending.to_string(), "Pending");
        assert_eq!(PodStatus::Succeeded.to_string(), "Succeeded");
        assert_eq!(PodStatus::Failed.to_string(), "Failed");
        assert_eq!(PodStatus::Unknown.to_string(), "Unknown");
        assert_eq!(PodStatus::CrashLoopBackOff.to_string(), "CrashLoop");
        assert_eq!(PodStatus::ContainerCreating.to_string(), "Creating");
        assert_eq!(PodStatus::Terminating.to_string(), "Terminating");
        assert_eq!(PodStatus::ImagePullBackOff.to_string(), "ImgPull");
    }

    #[test]
    fn namespace_filter_methods() {
        #[cfg(not(feature = "k8s"))]
        {
            let mut collector = K8sCollector::new();
            assert!(collector.namespace_filter().is_none());
            assert!(collector.available_namespaces().is_empty());

            collector.set_namespace_filter(Some("default".to_string()));
            assert_eq!(collector.namespace_filter(), Some("default"));

            collector.set_namespace_filter(None);
            assert!(collector.namespace_filter().is_none());
        }
    }
}
