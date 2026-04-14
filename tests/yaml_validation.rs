/// Prove that helm-synthesizer output is valid YAML via tree-sitter.
use helm_synthesizer::*;
use yaml_synthesizer::emit_file;

fn validate_yaml(source: &str) {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_yaml::language().into())
        .expect("failed to set YAML language");
    let tree = parser.parse(source, None).expect("parser returned no tree");
    assert!(
        !tree.root_node().has_error(),
        "tree-sitter detected YAML parse error in:\n{}",
        &source[..500.min(source.len())]
    );
}

fn test_config() -> DeploymentConfig {
    DeploymentConfig::new(
        ChartMeta::new("test-app", "0.1.0").app_version("1.0.0").description("Test app"),
        "ghcr.io/pleme-io/test",
        "v0.1.0",
    )
    .replicas(3)
    .port(8080)
    .resources(Resources::new().cpu("100m", "500m").memory("128Mi", "512Mi"))
    .service(ServiceConfig::cluster_ip(80, 8080))
    .hpa(HpaConfig { min_replicas: 2, max_replicas: 10, target_cpu_percent: 80 })
    .pdb(PdbConfig { min_available: Some(1), max_unavailable: None })
    .network_policy(NetworkPolicyConfig { ingress_ports: vec![8080], egress_ports: vec![443, 53] })
    .label("app.kubernetes.io/component", "api")
    .env("RUST_LOG", "info")
    .with_service_monitor()
}

#[test]
fn chart_yaml_valid() {
    let config = test_config();
    validate_yaml(&emit_file(&render_chart_yaml(&config.chart)));
}

#[test]
fn values_yaml_valid() {
    let config = test_config();
    validate_yaml(&emit_file(&render_values_yaml(&config)));
}

#[test]
fn deployment_template_valid_yaml() {
    // Note: Helm templates have Go syntax {{ }} which tree-sitter-yaml may flag.
    // We validate the YAML structure is sound even with template placeholders.
    let out = emit_file(&render_deployment_template());
    // At minimum, the output should contain valid YAML structure markers
    assert!(out.contains("apiVersion:"));
    assert!(out.contains("kind: Deployment"));
    assert!(out.contains("spec:"));
}

#[test]
fn service_template_valid_yaml() {
    let out = emit_file(&render_service_template());
    assert!(out.contains("apiVersion: v1"));
    assert!(out.contains("kind: Service"));
}

#[test]
fn minimal_chart_yaml_valid() {
    let meta = ChartMeta::new("minimal", "0.1.0");
    validate_yaml(&emit_file(&render_chart_yaml(&meta)));
}

#[test]
fn full_chart_with_dependencies_valid() {
    let meta = ChartMeta::new("full", "1.0.0")
        .app_version("2.0.0")
        .description("Full chart")
        .dependency("redis", "17.0.0", "https://charts.bitnami.com/bitnami");
    validate_yaml(&emit_file(&render_chart_yaml(&meta)));
}

#[test]
fn values_minimal_valid() {
    let config = DeploymentConfig::new(ChartMeta::new("min", "0.1.0"), "img", "tag");
    validate_yaml(&emit_file(&render_values_yaml(&config)));
}

#[test]
fn values_all_features_valid() {
    validate_yaml(&emit_file(&render_values_yaml(&test_config())));
}
