# helm-synthesizer

> **★★★ CSE / Knowable Construction.** This repo operates under **Constructive Substrate Engineering** — canonical specification at [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md). The Compounding Directive (operational rules: solve once, load-bearing fixes only, idiom-first, models stay current, direction beats velocity) is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before non-trivial changes.


Typed Helm chart generation from proven infrastructure types. Built on yaml-synthesizer. **Zero Raw nodes** — every Helm Go template expression is a typed `HelmExpr` variant.

## Tests: 59 | Status: Proven, tree-sitter Validated, Zero Raw

## HelmExpr — Typed Go Template Expressions

| Variant | Output |
|---------|--------|
| `Value(["service", "port"])` | `{{ .Values.service.port }}` |
| `Include("chart.fullname")` | `{{ include "chart.fullname" . }}` |
| `IncludeNindent("chart.labels", 4)` | `{{- include "chart.labels" . | nindent 4 }}` |
| `ChartField("Name")` | `{{ .Chart.Name }}` |
| `Interpolated([...])` | `"{{ .Values.image.repository }}:{{ .Values.image.tag }}"` |

No `YamlNode::Raw` anywhere. Every expression is typed and validatable.

## Core Types

| Type | Purpose |
|------|---------|
| `ChartMeta` | Chart.yaml: apiVersion, name, version, type, dependencies |
| `DeploymentConfig` | Root: image, replicas, port, resources, security, service, hpa, pdb, network policy |
| `SecurityContext` | `hardened()` (default) or `permissive()` |

## Rendering

- `render_chart_yaml(&ChartMeta)` → Chart.yaml
- `render_values_yaml(&DeploymentConfig)` → values.yaml
- `render_deployment_template()` → templates/deployment.yaml (HelmExpr, not Raw)
- `render_service_template()` → templates/service.yaml (HelmExpr, not Raw)

## tree-sitter Validation

8 tests validate Chart.yaml and values.yaml via `tree-sitter-yaml`.

## Dependencies

yaml-synthesizer (path). proptest, tree-sitter, tree-sitter-yaml (dev).
