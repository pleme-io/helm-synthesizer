use helm_synthesizer::*;
use yaml_synthesizer::emit_file;

// ── ChartMeta rendering proofs ──────────────────────────────────────

#[test]
fn chart_yaml_api_version_v2() {
    let meta = ChartMeta::new("app", "1.0.0");
    let out = emit_file(&render_chart_yaml(&meta));
    assert!(out.contains("apiVersion: v2"));
}

#[test]
fn chart_yaml_name_present() {
    let meta = ChartMeta::new("my-chart", "0.1.0");
    let out = emit_file(&render_chart_yaml(&meta));
    assert!(out.contains("name: my-chart"));
}

#[test]
fn chart_yaml_version_present() {
    let meta = ChartMeta::new("app", "3.2.1");
    let out = emit_file(&render_chart_yaml(&meta));
    assert!(out.contains("version: 3.2.1"));
}

#[test]
fn chart_yaml_type_application_default() {
    let meta = ChartMeta::new("app", "1.0.0");
    let out = emit_file(&render_chart_yaml(&meta));
    assert!(out.contains("type: application"));
}

#[test]
fn chart_yaml_type_library() {
    let meta = ChartMeta::new("lib", "1.0.0").library();
    let out = emit_file(&render_chart_yaml(&meta));
    assert!(out.contains("type: library"));
}

#[test]
fn chart_yaml_app_version() {
    let meta = ChartMeta::new("app", "1.0.0").app_version("2.0.0");
    let out = emit_file(&render_chart_yaml(&meta));
    assert!(out.contains("appVersion: 2.0.0"));
}

#[test]
fn chart_yaml_description() {
    let meta = ChartMeta::new("app", "1.0.0").description("My application");
    let out = emit_file(&render_chart_yaml(&meta));
    assert!(out.contains("description: My application"));
}

#[test]
fn chart_yaml_dependencies() {
    let meta = ChartMeta::new("app", "1.0.0")
        .dependency("redis", "17.0.0", "https://charts.bitnami.com/bitnami");
    let out = emit_file(&render_chart_yaml(&meta));
    assert!(out.contains("dependencies:"));
    assert!(out.contains("name: redis"));
    assert!(out.contains("version: 17.0.0"));
}

// ── DeploymentConfig values.yaml proofs ─────────────────────────────

fn test_config() -> DeploymentConfig {
    DeploymentConfig::new(
        ChartMeta::new("test", "0.1.0"),
        "ghcr.io/pleme-io/test",
        "v0.1.0",
    )
}

#[test]
fn values_replica_count() {
    let config = test_config().replicas(5);
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("replicaCount: 5"));
}

#[test]
fn values_image_repository() {
    let out = emit_file(&render_values_yaml(&test_config()));
    assert!(out.contains("repository: ghcr.io/pleme-io/test"));
}

#[test]
fn values_image_tag() {
    let out = emit_file(&render_values_yaml(&test_config()));
    assert!(out.contains("tag: v0.1.0"));
}

#[test]
fn values_image_pull_policy() {
    let out = emit_file(&render_values_yaml(&test_config()));
    assert!(out.contains("pullPolicy: IfNotPresent"));
}

#[test]
fn values_container_port() {
    let config = test_config().port(3000);
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("containerPort: 3000"));
}

#[test]
fn values_service_cluster_ip() {
    let config = test_config().service(ServiceConfig::cluster_ip(80, 8080));
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("type: ClusterIP"));
    assert!(out.contains("port: 80"));
    assert!(out.contains("targetPort: 8080"));
}

#[test]
fn values_service_load_balancer() {
    let config = test_config().service(ServiceConfig::load_balancer(443, 8443));
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("type: LoadBalancer"));
}

#[test]
fn values_resources() {
    let config = test_config().resources(
        Resources::new()
            .cpu("100m", "500m")
            .memory("128Mi", "512Mi"),
    );
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("cpu: 100m"));
    assert!(out.contains("memory: 128Mi"));
    assert!(out.contains("cpu: 500m"));
    assert!(out.contains("memory: 512Mi"));
}

#[test]
fn values_security_hardened_default() {
    let out = emit_file(&render_values_yaml(&test_config()));
    assert!(out.contains("runAsNonRoot: true"));
    assert!(out.contains("readOnlyRootFilesystem: true"));
    assert!(out.contains("ALL"));
}

#[test]
fn values_security_permissive() {
    let config = test_config().security(SecurityContext::permissive());
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("runAsNonRoot: false"));
}

#[test]
fn values_hpa() {
    let config = test_config().hpa(HpaConfig {
        min_replicas: 2,
        max_replicas: 10,
        target_cpu_percent: 80,
    });
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("autoscaling:"));
    assert!(out.contains("enabled: true"));
    assert!(out.contains("minReplicas: 2"));
    assert!(out.contains("maxReplicas: 10"));
    assert!(out.contains("targetCPUUtilizationPercentage: 80"));
}

#[test]
fn values_pdb() {
    let config = test_config().pdb(PdbConfig {
        min_available: Some(1),
        max_unavailable: None,
    });
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("podDisruptionBudget:"));
    assert!(out.contains("minAvailable: 1"));
}

#[test]
fn values_network_policy() {
    let config = test_config().network_policy(NetworkPolicyConfig {
        ingress_ports: vec![8080],
        egress_ports: vec![443, 53],
    });
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("networkPolicy:"));
    assert!(out.contains("enabled: true"));
}

#[test]
fn values_labels() {
    let config = test_config()
        .label("app.kubernetes.io/component", "api")
        .label("team", "platform");
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("labels:"));
    assert!(out.contains("app.kubernetes.io/component: api"));
}

#[test]
fn values_env() {
    let config = test_config()
        .env("RUST_LOG", "info")
        .env("PORT", "8080");
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("RUST_LOG"));
    assert!(out.contains("info"));
}

#[test]
fn values_service_monitor() {
    let config = test_config().with_service_monitor();
    let out = emit_file(&render_values_yaml(&config));
    assert!(out.contains("serviceMonitor:"));
    assert!(out.contains("enabled: true"));
}

// ── Template rendering proofs ───────────────────────────────────────

#[test]
fn deployment_template_api_version() {
    let out = emit_file(&render_deployment_template());
    assert!(out.contains("apiVersion: apps/v1"));
}

#[test]
fn deployment_template_kind() {
    let out = emit_file(&render_deployment_template());
    assert!(out.contains("kind: Deployment"));
}

#[test]
fn deployment_template_has_spec() {
    let out = emit_file(&render_deployment_template());
    assert!(out.contains("spec:"));
    assert!(out.contains("replicas:"));
    assert!(out.contains("containers:"));
}

#[test]
fn deployment_template_uses_helm_values() {
    let out = emit_file(&render_deployment_template());
    assert!(out.contains(".Values.replicaCount"));
    assert!(out.contains(".Values.image.repository"));
    assert!(out.contains(".Values.containerPort"));
}

#[test]
fn deployment_template_uses_helpers() {
    let out = emit_file(&render_deployment_template());
    assert!(out.contains("chart.fullname"));
    assert!(out.contains("chart.labels"));
    assert!(out.contains("chart.selectorLabels"));
}

#[test]
fn service_template_api_version() {
    let out = emit_file(&render_service_template());
    assert!(out.contains("apiVersion: v1"));
}

#[test]
fn service_template_kind() {
    let out = emit_file(&render_service_template());
    assert!(out.contains("kind: Service"));
}

#[test]
fn service_template_uses_helm_values() {
    let out = emit_file(&render_service_template());
    assert!(out.contains(".Values.service.type"));
    assert!(out.contains(".Values.service.port"));
}

// ── Determinism proofs ──────────────────────────────────────────────

#[test]
fn chart_yaml_deterministic() {
    let meta = ChartMeta::new("app", "1.0.0").description("test");
    let a = emit_file(&render_chart_yaml(&meta));
    let b = emit_file(&render_chart_yaml(&meta));
    assert_eq!(a, b);
}

#[test]
fn values_yaml_deterministic() {
    let config = test_config()
        .replicas(3)
        .resources(Resources::new().cpu("100m", "500m"))
        .service(ServiceConfig::cluster_ip(80, 8080));
    let a = emit_file(&render_values_yaml(&config));
    let b = emit_file(&render_values_yaml(&config));
    assert_eq!(a, b);
}

#[test]
fn deployment_template_deterministic() {
    let a = emit_file(&render_deployment_template());
    let b = emit_file(&render_deployment_template());
    assert_eq!(a, b);
}

// ── Realistic full chart proof ──────────────────────────────────────

#[test]
fn realistic_vector_chart() {
    let config = DeploymentConfig::new(
        ChartMeta::new("pleme-vector", "0.2.8")
            .app_version("0.41.1")
            .description("Vector log aggregator for Shinryu data platform"),
        "timberio/vector",
        "0.41.1-distroless-libc",
    )
    .replicas(2)
    .port(8686)
    .resources(Resources::new().cpu("200m", "1000m").memory("256Mi", "1Gi"))
    .security(SecurityContext::hardened())
    .service(ServiceConfig::cluster_ip(8686, 8686))
    .hpa(HpaConfig {
        min_replicas: 2,
        max_replicas: 5,
        target_cpu_percent: 80,
    })
    .pdb(PdbConfig {
        min_available: Some(1),
        max_unavailable: None,
    })
    .network_policy(NetworkPolicyConfig {
        ingress_ports: vec![8686],
        egress_ports: vec![443, 9200],
    })
    .label("app.kubernetes.io/component", "observability")
    .with_service_monitor();

    let chart_yaml = emit_file(&render_chart_yaml(&config.chart));
    let values_yaml = emit_file(&render_values_yaml(&config));

    // Chart.yaml
    assert!(chart_yaml.contains("name: pleme-vector"));
    assert!(chart_yaml.contains("version: 0.2.8"));
    assert!(chart_yaml.contains("appVersion: 0.41.1"));

    // Values
    assert!(values_yaml.contains("replicaCount: 2"));
    assert!(values_yaml.contains("timberio/vector"));
    assert!(values_yaml.contains("0.41.1-distroless-libc"));
    assert!(values_yaml.contains("containerPort: 8686"));
    assert!(values_yaml.contains("runAsNonRoot: true"));
    assert!(values_yaml.contains("serviceMonitor:"));
    assert!(values_yaml.contains("networkPolicy:"));
    assert!(values_yaml.contains("podDisruptionBudget:"));
    assert!(values_yaml.contains("autoscaling:"));
}
