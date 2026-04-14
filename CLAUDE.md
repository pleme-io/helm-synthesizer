# helm-synthesizer

Typed Helm chart generation from proven infrastructure types. Built on yaml-synthesizer.

## Tests: 44 | Status: Proven

## Core Types

| Type | Purpose |
|------|---------|
| `ChartMeta` | Chart.yaml: apiVersion, name, version, type, dependencies |
| `DeploymentConfig` | Root type: image, replicas, port, resources, security, service, hpa, pdb, network policy, labels, env, service monitor |
| `Resources` | CPU/memory requests and limits |
| `SecurityContext` | `hardened()` (default) or `permissive()` |
| `ServiceConfig` | ClusterIP / NodePort / LoadBalancer |
| `HpaConfig` | min/max replicas, target CPU |
| `PdbConfig` | minAvailable / maxUnavailable |
| `NetworkPolicyConfig` | ingress/egress ports |

## Rendering

- `render_chart_yaml(&ChartMeta)` → Chart.yaml as YamlNode
- `render_values_yaml(&DeploymentConfig)` → values.yaml as YamlNode
- `render_deployment_template()` → templates/deployment.yaml with Helm Go template syntax
- `render_service_template()` → templates/service.yaml

## Security Default

All charts default to `SecurityContext::hardened()`: runAsNonRoot=true, readOnlyRootFilesystem=true, drop ALL capabilities.

## Dependencies

yaml-synthesizer (path dep). proptest (dev).
