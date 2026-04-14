/// Typed Helm chart configuration — proven types that generate
/// structurally correct Helm charts.

/// Chart.yaml metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartMeta {
    pub api_version: ChartApiVersion,
    pub name: String,
    pub version: String,
    pub app_version: Option<String>,
    pub description: Option<String>,
    pub chart_type: ChartType,
    pub dependencies: Vec<ChartDependency>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChartApiVersion {
    V2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChartType {
    Application,
    Library,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartDependency {
    pub name: String,
    pub version: String,
    pub repository: String,
    pub condition: Option<String>,
}

/// Container resource limits/requests.
#[derive(Debug, Clone, PartialEq)]
pub struct Resources {
    pub cpu_request: Option<String>,
    pub memory_request: Option<String>,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
}

/// Container security context.
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityContext {
    pub run_as_non_root: bool,
    pub read_only_root_filesystem: bool,
    pub drop_capabilities: Vec<String>,
}

/// Service configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceConfig {
    pub service_type: ServiceType,
    pub port: u16,
    pub target_port: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
}

/// HPA configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct HpaConfig {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_percent: u32,
}

/// PDB configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct PdbConfig {
    pub min_available: Option<u32>,
    pub max_unavailable: Option<u32>,
}

/// NetworkPolicy configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkPolicyConfig {
    pub ingress_ports: Vec<u16>,
    pub egress_ports: Vec<u16>,
}

/// Complete deployment configuration — the root proven type.
#[derive(Debug, Clone, PartialEq)]
pub struct DeploymentConfig {
    pub chart: ChartMeta,
    pub image: String,
    pub image_tag: String,
    pub replicas: u32,
    pub container_port: u16,
    pub resources: Resources,
    pub security_context: SecurityContext,
    pub service: Option<ServiceConfig>,
    pub hpa: Option<HpaConfig>,
    pub pdb: Option<PdbConfig>,
    pub network_policy: Option<NetworkPolicyConfig>,
    pub labels: Vec<(String, String)>,
    pub env: Vec<(String, String)>,
    pub service_monitor: bool,
}

// ── Constructors ────────────────────────────────────────────────────

impl ChartMeta {
    #[must_use]
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            api_version: ChartApiVersion::V2,
            name: name.to_string(),
            version: version.to_string(),
            app_version: None,
            description: None,
            chart_type: ChartType::Application,
            dependencies: Vec::new(),
        }
    }

    #[must_use]
    pub fn app_version(mut self, v: &str) -> Self {
        self.app_version = Some(v.to_string());
        self
    }

    #[must_use]
    pub fn description(mut self, d: &str) -> Self {
        self.description = Some(d.to_string());
        self
    }

    #[must_use]
    pub fn library(mut self) -> Self {
        self.chart_type = ChartType::Library;
        self
    }

    #[must_use]
    pub fn dependency(mut self, name: &str, version: &str, repo: &str) -> Self {
        self.dependencies.push(ChartDependency {
            name: name.to_string(),
            version: version.to_string(),
            repository: repo.to_string(),
            condition: None,
        });
        self
    }
}

impl Resources {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cpu_request: None,
            memory_request: None,
            cpu_limit: None,
            memory_limit: None,
        }
    }

    #[must_use]
    pub fn cpu(mut self, request: &str, limit: &str) -> Self {
        self.cpu_request = Some(request.to_string());
        self.cpu_limit = Some(limit.to_string());
        self
    }

    #[must_use]
    pub fn memory(mut self, request: &str, limit: &str) -> Self {
        self.memory_request = Some(request.to_string());
        self.memory_limit = Some(limit.to_string());
        self
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityContext {
    #[must_use]
    pub fn hardened() -> Self {
        Self {
            run_as_non_root: true,
            read_only_root_filesystem: true,
            drop_capabilities: vec!["ALL".to_string()],
        }
    }

    #[must_use]
    pub fn permissive() -> Self {
        Self {
            run_as_non_root: false,
            read_only_root_filesystem: false,
            drop_capabilities: Vec::new(),
        }
    }
}

impl DeploymentConfig {
    #[must_use]
    pub fn new(chart: ChartMeta, image: &str, tag: &str) -> Self {
        Self {
            chart,
            image: image.to_string(),
            image_tag: tag.to_string(),
            replicas: 1,
            container_port: 8080,
            resources: Resources::new(),
            security_context: SecurityContext::hardened(),
            service: None,
            hpa: None,
            pdb: None,
            network_policy: None,
            labels: Vec::new(),
            env: Vec::new(),
            service_monitor: false,
        }
    }

    #[must_use]
    pub fn replicas(mut self, n: u32) -> Self {
        self.replicas = n;
        self
    }

    #[must_use]
    pub fn port(mut self, p: u16) -> Self {
        self.container_port = p;
        self
    }

    #[must_use]
    pub fn resources(mut self, r: Resources) -> Self {
        self.resources = r;
        self
    }

    #[must_use]
    pub fn security(mut self, s: SecurityContext) -> Self {
        self.security_context = s;
        self
    }

    #[must_use]
    pub fn service(mut self, svc: ServiceConfig) -> Self {
        self.service = Some(svc);
        self
    }

    #[must_use]
    pub fn hpa(mut self, h: HpaConfig) -> Self {
        self.hpa = Some(h);
        self
    }

    #[must_use]
    pub fn pdb(mut self, p: PdbConfig) -> Self {
        self.pdb = Some(p);
        self
    }

    #[must_use]
    pub fn network_policy(mut self, np: NetworkPolicyConfig) -> Self {
        self.network_policy = Some(np);
        self
    }

    #[must_use]
    pub fn label(mut self, key: &str, value: &str) -> Self {
        self.labels.push((key.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn with_service_monitor(mut self) -> Self {
        self.service_monitor = true;
        self
    }
}

impl ServiceConfig {
    #[must_use]
    pub fn cluster_ip(port: u16, target_port: u16) -> Self {
        Self {
            service_type: ServiceType::ClusterIP,
            port,
            target_port,
        }
    }

    #[must_use]
    pub fn load_balancer(port: u16, target_port: u16) -> Self {
        Self {
            service_type: ServiceType::LoadBalancer,
            port,
            target_port,
        }
    }
}

impl ServiceType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClusterIP => "ClusterIP",
            Self::NodePort => "NodePort",
            Self::LoadBalancer => "LoadBalancer",
        }
    }
}

impl ChartApiVersion {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V2 => "v2",
        }
    }
}

impl ChartType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Library => "library",
        }
    }
}
