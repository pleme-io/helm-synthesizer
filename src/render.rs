use yaml_synthesizer::{YamlEntry, YamlNode};

use crate::types::*;

/// Render Chart.yaml from proven `ChartMeta` type.
#[must_use]
pub fn render_chart_yaml(meta: &ChartMeta) -> YamlNode {
    let mut entries = vec![
        YamlEntry::new("apiVersion", YamlNode::str(meta.api_version.as_str())),
        YamlEntry::new("name", YamlNode::str(&meta.name)),
        YamlEntry::new("version", YamlNode::str(&meta.version)),
        YamlEntry::new("type", YamlNode::str(meta.chart_type.as_str())),
    ];

    if let Some(ref av) = meta.app_version {
        entries.push(YamlEntry::new("appVersion", YamlNode::str(av)));
    }
    if let Some(ref desc) = meta.description {
        entries.push(YamlEntry::new("description", YamlNode::str(desc)));
    }

    if !meta.dependencies.is_empty() {
        let deps: Vec<YamlNode> = meta
            .dependencies
            .iter()
            .map(|d| {
                let mut dep_entries = vec![
                    YamlEntry::new("name", YamlNode::str(&d.name)),
                    YamlEntry::new("version", YamlNode::str(&d.version)),
                    YamlEntry::new("repository", YamlNode::str(&d.repository)),
                ];
                if let Some(ref cond) = d.condition {
                    dep_entries.push(YamlEntry::new("condition", YamlNode::str(cond)));
                }
                YamlNode::Map(dep_entries)
            })
            .collect();
        entries.push(YamlEntry::new("dependencies", YamlNode::Seq(deps)));
    }

    YamlNode::Map(entries)
}

/// Render values.yaml from proven `DeploymentConfig` type.
#[must_use]
pub fn render_values_yaml(config: &DeploymentConfig) -> YamlNode {
    let mut entries = vec![
        YamlEntry::new("replicaCount", YamlNode::Int(config.replicas.into())),
    ];

    // Image
    entries.push(YamlEntry::new(
        "image",
        YamlNode::map(vec![
            ("repository", YamlNode::str(&config.image)),
            ("tag", YamlNode::str(&config.image_tag)),
            ("pullPolicy", YamlNode::str("IfNotPresent")),
        ]),
    ));

    // Service
    if let Some(ref svc) = config.service {
        entries.push(YamlEntry::new(
            "service",
            YamlNode::map(vec![
                ("type", YamlNode::str(svc.service_type.as_str())),
                ("port", YamlNode::Int(svc.port.into())),
                ("targetPort", YamlNode::Int(svc.target_port.into())),
            ]),
        ));
    }

    // Resources
    let mut res_entries = Vec::new();
    let mut req_entries = Vec::new();
    let mut lim_entries = Vec::new();

    if let Some(ref cpu) = config.resources.cpu_request {
        req_entries.push(YamlEntry::new("cpu", YamlNode::str(cpu)));
    }
    if let Some(ref mem) = config.resources.memory_request {
        req_entries.push(YamlEntry::new("memory", YamlNode::str(mem)));
    }
    if let Some(ref cpu) = config.resources.cpu_limit {
        lim_entries.push(YamlEntry::new("cpu", YamlNode::str(cpu)));
    }
    if let Some(ref mem) = config.resources.memory_limit {
        lim_entries.push(YamlEntry::new("memory", YamlNode::str(mem)));
    }
    if !req_entries.is_empty() {
        res_entries.push(YamlEntry::new("requests", YamlNode::Map(req_entries)));
    }
    if !lim_entries.is_empty() {
        res_entries.push(YamlEntry::new("limits", YamlNode::Map(lim_entries)));
    }
    if !res_entries.is_empty() {
        entries.push(YamlEntry::new("resources", YamlNode::Map(res_entries)));
    }

    // Security context
    let sec = &config.security_context;
    let mut sec_entries = vec![
        YamlEntry::new("runAsNonRoot", YamlNode::Bool(sec.run_as_non_root)),
        YamlEntry::new(
            "readOnlyRootFilesystem",
            YamlNode::Bool(sec.read_only_root_filesystem),
        ),
    ];
    if !sec.drop_capabilities.is_empty() {
        sec_entries.push(YamlEntry::new(
            "capabilities",
            YamlNode::map(vec![(
                "drop",
                YamlNode::Seq(sec.drop_capabilities.iter().map(|c| YamlNode::str(c)).collect()),
            )]),
        ));
    }
    entries.push(YamlEntry::new("securityContext", YamlNode::Map(sec_entries)));

    // HPA
    if let Some(ref hpa) = config.hpa {
        entries.push(YamlEntry::new(
            "autoscaling",
            YamlNode::map(vec![
                ("enabled", YamlNode::Bool(true)),
                ("minReplicas", YamlNode::Int(hpa.min_replicas.into())),
                ("maxReplicas", YamlNode::Int(hpa.max_replicas.into())),
                (
                    "targetCPUUtilizationPercentage",
                    YamlNode::Int(hpa.target_cpu_percent.into()),
                ),
            ]),
        ));
    }

    // PDB
    if let Some(ref pdb) = config.pdb {
        let mut pdb_entries = vec![YamlEntry::new("enabled", YamlNode::Bool(true))];
        if let Some(min) = pdb.min_available {
            pdb_entries.push(YamlEntry::new("minAvailable", YamlNode::Int(min.into())));
        }
        if let Some(max) = pdb.max_unavailable {
            pdb_entries.push(YamlEntry::new("maxUnavailable", YamlNode::Int(max.into())));
        }
        entries.push(YamlEntry::new(
            "podDisruptionBudget",
            YamlNode::Map(pdb_entries),
        ));
    }

    // Network policy
    if let Some(ref np) = config.network_policy {
        entries.push(YamlEntry::new(
            "networkPolicy",
            YamlNode::map(vec![
                ("enabled", YamlNode::Bool(true)),
                (
                    "ingressPorts",
                    YamlNode::Seq(np.ingress_ports.iter().map(|p| YamlNode::Int((*p).into())).collect()),
                ),
                (
                    "egressPorts",
                    YamlNode::Seq(np.egress_ports.iter().map(|p| YamlNode::Int((*p).into())).collect()),
                ),
            ]),
        ));
    }

    // Labels
    if !config.labels.is_empty() {
        let label_entries: Vec<YamlEntry> = config
            .labels
            .iter()
            .map(|(k, v)| YamlEntry::new(k, YamlNode::str(v)))
            .collect();
        entries.push(YamlEntry::new("labels", YamlNode::Map(label_entries)));
    }

    // Env
    if !config.env.is_empty() {
        let env_seq: Vec<YamlNode> = config
            .env
            .iter()
            .map(|(k, v)| {
                YamlNode::map(vec![("name", YamlNode::str(k)), ("value", YamlNode::str(v))])
            })
            .collect();
        entries.push(YamlEntry::new("env", YamlNode::Seq(env_seq)));
    }

    // ServiceMonitor
    if config.service_monitor {
        entries.push(YamlEntry::new(
            "serviceMonitor",
            YamlNode::map(vec![("enabled", YamlNode::Bool(true))]),
        ));
    }

    entries.push(YamlEntry::new(
        "containerPort",
        YamlNode::Int(config.container_port.into()),
    ));

    YamlNode::Map(entries)
}

/// Render a Deployment template (templates/deployment.yaml).
#[must_use]
pub fn render_deployment_template() -> YamlNode {
    // This renders the Go template YAML that Helm processes
    YamlNode::map(vec![
        ("apiVersion", YamlNode::str("apps/v1")),
        ("kind", YamlNode::str("Deployment")),
        (
            "metadata",
            YamlNode::map(vec![
                ("name", YamlNode::Raw("{{ include \"chart.fullname\" . }}".into())),
                ("labels", YamlNode::Raw("{{- include \"chart.labels\" . | nindent 4 }}".into())),
            ]),
        ),
        (
            "spec",
            YamlNode::map(vec![
                (
                    "replicas",
                    YamlNode::Raw("{{ .Values.replicaCount }}".into()),
                ),
                (
                    "selector",
                    YamlNode::map(vec![(
                        "matchLabels",
                        YamlNode::Raw("{{- include \"chart.selectorLabels\" . | nindent 6 }}".into()),
                    )]),
                ),
                (
                    "template",
                    YamlNode::map(vec![
                        (
                            "metadata",
                            YamlNode::map(vec![(
                                "labels",
                                YamlNode::Raw(
                                    "{{- include \"chart.selectorLabels\" . | nindent 8 }}".into(),
                                ),
                            )]),
                        ),
                        (
                            "spec",
                            YamlNode::map(vec![(
                                "containers",
                                YamlNode::Seq(vec![YamlNode::map(vec![
                                    ("name", YamlNode::Raw("{{ .Chart.Name }}".into())),
                                    (
                                        "image",
                                        YamlNode::Raw(
                                            "\"{{ .Values.image.repository }}:{{ .Values.image.tag }}\"".into(),
                                        ),
                                    ),
                                    (
                                        "ports",
                                        YamlNode::Seq(vec![YamlNode::map(vec![(
                                            "containerPort",
                                            YamlNode::Raw("{{ .Values.containerPort }}".into()),
                                        )])]),
                                    ),
                                ])]),
                            )]),
                        ),
                    ]),
                ),
            ]),
        ),
    ])
}

/// Render a Service template.
#[must_use]
pub fn render_service_template() -> YamlNode {
    YamlNode::map(vec![
        ("apiVersion", YamlNode::str("v1")),
        ("kind", YamlNode::str("Service")),
        (
            "metadata",
            YamlNode::map(vec![
                ("name", YamlNode::Raw("{{ include \"chart.fullname\" . }}".into())),
                ("labels", YamlNode::Raw("{{- include \"chart.labels\" . | nindent 4 }}".into())),
            ]),
        ),
        (
            "spec",
            YamlNode::map(vec![
                ("type", YamlNode::Raw("{{ .Values.service.type }}".into())),
                (
                    "ports",
                    YamlNode::Seq(vec![YamlNode::map(vec![
                        ("port", YamlNode::Raw("{{ .Values.service.port }}".into())),
                        (
                            "targetPort",
                            YamlNode::Raw("{{ .Values.service.targetPort }}".into()),
                        ),
                    ])]),
                ),
                (
                    "selector",
                    YamlNode::Raw("{{- include \"chart.selectorLabels\" . | nindent 4 }}".into()),
                ),
            ]),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_synthesizer::emit_file;

    #[test]
    fn chart_yaml_has_api_version() {
        let meta = ChartMeta::new("test-chart", "0.1.0");
        let out = emit_file(&render_chart_yaml(&meta));
        assert!(out.contains("apiVersion: v2"));
    }

    #[test]
    fn chart_yaml_has_name() {
        let meta = ChartMeta::new("my-app", "1.0.0");
        let out = emit_file(&render_chart_yaml(&meta));
        assert!(out.contains("name: my-app"));
    }

    #[test]
    fn chart_yaml_has_version() {
        let meta = ChartMeta::new("app", "2.3.4");
        let out = emit_file(&render_chart_yaml(&meta));
        assert!(out.contains("version: 2.3.4"));
    }

    #[test]
    fn values_yaml_has_replica_count() {
        let config = DeploymentConfig::new(
            ChartMeta::new("app", "1.0.0"),
            "ghcr.io/org/app",
            "latest",
        )
        .replicas(3);
        let out = emit_file(&render_values_yaml(&config));
        assert!(out.contains("replicaCount: 3"));
    }

    #[test]
    fn values_yaml_has_image() {
        let config = DeploymentConfig::new(
            ChartMeta::new("app", "1.0.0"),
            "ghcr.io/org/app",
            "v1.2.3",
        );
        let out = emit_file(&render_values_yaml(&config));
        assert!(out.contains("ghcr.io/org/app"));
        assert!(out.contains("v1.2.3"));
    }

    #[test]
    fn values_yaml_hardened_security() {
        let config = DeploymentConfig::new(
            ChartMeta::new("app", "1.0.0"),
            "img",
            "tag",
        );
        let out = emit_file(&render_values_yaml(&config));
        assert!(out.contains("runAsNonRoot: true"));
        assert!(out.contains("readOnlyRootFilesystem: true"));
        assert!(out.contains("ALL"));
    }

    #[test]
    fn deployment_template_has_kind() {
        let out = emit_file(&render_deployment_template());
        assert!(out.contains("kind: Deployment"));
        assert!(out.contains("apiVersion: apps/v1"));
    }

    #[test]
    fn service_template_has_kind() {
        let out = emit_file(&render_service_template());
        assert!(out.contains("kind: Service"));
    }
}
