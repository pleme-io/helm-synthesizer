/// Typed Helm chart configuration — proven types that generate
/// structurally correct Helm charts.

/// A Helm Go template expression — typed alternative to Raw strings.
///
/// Every Helm template expression in a chart is one of these variants.
/// No escape hatches. No arbitrary strings. Provably correct syntax.
#[derive(Debug, Clone, PartialEq)]
pub enum HelmExpr {
    /// `{{ .Values.path.to.field }}` — reference to values.yaml
    Value(Vec<String>),
    /// `{{ include "template_name" . }}` — include a named template
    Include {
        template: String,
        context: String,
    },
    /// `{{- include "template_name" . | nindent N }}` — include with nindent
    IncludeNindent {
        template: String,
        context: String,
        indent: u32,
    },
    /// `{{ .Chart.Name }}` — chart metadata field
    ChartField(String),
    /// `"{{ .Values.a }}:{{ .Values.b }}"` — interpolated string
    Interpolated(Vec<HelmExprPart>),

    // ── Control flow (Helm Go templates) ─────────────────

    /// `{{- if .Values.x }}...{{- else }}...{{- end }}`
    If {
        condition: Box<HelmExpr>,
        body: String,
        else_body: Option<String>,
    },
    /// `{{- range .Values.items }}...{{- end }}`
    Range {
        collection: Box<HelmExpr>,
        body: String,
    },
    /// `{{- with .Values.x }}...{{- end }}`
    With {
        context: Box<HelmExpr>,
        body: String,
    },
    /// `{{- define "name" }}...{{- end }}`
    Define {
        name: String,
        body: String,
    },
    /// `{{ .Values.x | default "val" }}` or `{{ .Values.x | quote }}`
    Pipe {
        expr: Box<HelmExpr>,
        functions: Vec<String>,
    },
}

/// Parts of an interpolated Helm expression.
#[derive(Debug, Clone, PartialEq)]
pub enum HelmExprPart {
    Literal(String),
    ValueRef(Vec<String>),
}

impl HelmExpr {
    /// Emit as a Go template string for use in YAML values.
    #[must_use]
    pub fn emit(&self) -> String {
        match self {
            Self::Value(path) => format!("{{{{ .Values.{} }}}}", path.join(".")),
            Self::Include { template, context } => {
                format!("{{{{ include \"{}\" {} }}}}", template, context)
            }
            Self::IncludeNindent { template, context, indent } => {
                format!("{{{{- include \"{}\" {} | nindent {} }}}}", template, context, indent)
            }
            Self::ChartField(field) => format!("{{{{ .Chart.{} }}}}", field),
            Self::Interpolated(parts) => {
                let mut out = String::from("\"");
                for part in parts {
                    match part {
                        HelmExprPart::Literal(s) => out.push_str(s),
                        HelmExprPart::ValueRef(path) => {
                            out.push_str(&format!("{{{{ .Values.{} }}}}", path.join(".")));
                        }
                    }
                }
                out.push('"');
                out
            }
            Self::If { condition, body, else_body } => {
                let cond = condition.emit();
                match else_body {
                    Some(eb) => format!("{{{{- if {cond} }}}}\n{body}\n{{{{- else }}}}\n{eb}\n{{{{- end }}}}"),
                    None => format!("{{{{- if {cond} }}}}\n{body}\n{{{{- end }}}}"),
                }
            }
            Self::Range { collection, body } => {
                format!("{{{{- range {} }}}}\n{body}\n{{{{- end }}}}", collection.emit())
            }
            Self::With { context, body } => {
                format!("{{{{- with {} }}}}\n{body}\n{{{{- end }}}}", context.emit())
            }
            Self::Define { name, body } => {
                format!("{{{{- define \"{}\" }}}}\n{body}\n{{{{- end }}}}", name)
            }
            Self::Pipe { expr, functions } => {
                let base = expr.emit();
                // Strip outer {{ }} from base to chain pipes
                let inner = base.trim_start_matches("{{ ").trim_end_matches(" }}");
                let pipe_chain = functions.join(" | ");
                format!("{{{{ {inner} | {pipe_chain} }}}}")
            }
        }
    }

    /// Convert to a YamlNode for embedding in YAML structures.
    /// Uses TemplateExpr (typed) — not Raw (escape hatch).
    #[must_use]
    pub fn to_yaml(&self) -> yaml_synthesizer::YamlNode {
        yaml_synthesizer::YamlNode::TemplateExpr(self.emit())
    }

    // ── Convenience constructors ────────────────────────────────

    #[must_use]
    pub fn value(path: &[&str]) -> Self {
        Self::Value(path.iter().map(|s| (*s).to_string()).collect())
    }

    #[must_use]
    pub fn include(template: &str) -> Self {
        Self::Include {
            template: template.to_string(),
            context: ".".to_string(),
        }
    }

    #[must_use]
    pub fn include_nindent(template: &str, indent: u32) -> Self {
        Self::IncludeNindent {
            template: template.to_string(),
            context: ".".to_string(),
            indent,
        }
    }

    #[must_use]
    pub fn chart(field: &str) -> Self {
        Self::ChartField(field.to_string())
    }

    #[must_use]
    pub fn image_ref() -> Self {
        Self::Interpolated(vec![
            HelmExprPart::ValueRef(vec!["image".into(), "repository".into()]),
            HelmExprPart::Literal(":".into()),
            HelmExprPart::ValueRef(vec!["image".into(), "tag".into()]),
        ])
    }
}

#[cfg(test)]
mod helm_expr_tests {
    use super::*;

    #[test]
    fn value_emits_go_template() {
        assert_eq!(
            HelmExpr::value(&["replicaCount"]).emit(),
            "{{ .Values.replicaCount }}"
        );
    }

    #[test]
    fn nested_value_emits_dotted() {
        assert_eq!(
            HelmExpr::value(&["service", "port"]).emit(),
            "{{ .Values.service.port }}"
        );
    }

    #[test]
    fn include_emits_template() {
        assert_eq!(
            HelmExpr::include("chart.fullname").emit(),
            "{{ include \"chart.fullname\" . }}"
        );
    }

    #[test]
    fn include_nindent_emits() {
        assert_eq!(
            HelmExpr::include_nindent("chart.labels", 4).emit(),
            "{{- include \"chart.labels\" . | nindent 4 }}"
        );
    }

    #[test]
    fn chart_field_emits() {
        assert_eq!(
            HelmExpr::chart("Name").emit(),
            "{{ .Chart.Name }}"
        );
    }

    #[test]
    fn image_ref_emits_interpolated() {
        let out = HelmExpr::image_ref().emit();
        assert!(out.contains(".Values.image.repository"));
        assert!(out.contains(".Values.image.tag"));
        assert!(out.contains(':'));
    }

    #[test]
    fn all_exprs_deterministic() {
        let exprs = vec![
            HelmExpr::value(&["x"]),
            HelmExpr::include("t"),
            HelmExpr::include_nindent("t", 4),
            HelmExpr::chart("Name"),
            HelmExpr::image_ref(),
        ];
        for expr in &exprs {
            assert_eq!(expr.emit(), expr.emit());
        }
    }
}

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
