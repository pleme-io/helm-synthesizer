//! Integration tests proving `HelmExpr` conforms to `synthesizer_core` traits.
//!
//! Wave 2 of the compound-knowledge refactor. Every test calls one of
//! `synthesizer_core::node::laws::*` on a real `HelmExpr` value, compounding
//! proof surface: the same laws prove properties of every synthesizer that
//! conforms.

use helm_synthesizer::{HelmExpr, HelmExprPart};
use synthesizer_core::node::laws;
use synthesizer_core::{NoRawAttestation, SynthesizerNode};

// ─── Trait shape ────────────────────────────────────────────────────

#[test]
fn indent_unit_is_two_spaces() {
    assert_eq!(<HelmExpr as SynthesizerNode>::indent_unit(), "  ");
}

#[test]
fn variant_ids_distinct_across_disjoint_variants() {
    let samples: Vec<HelmExpr> = vec![
        HelmExpr::value(&["replicaCount"]),
        HelmExpr::include("chart.fullname"),
        HelmExpr::include_nindent("chart.labels", 4),
        HelmExpr::chart("Name"),
        HelmExpr::image_ref(),
        HelmExpr::If {
            condition: Box::new(HelmExpr::value(&["enabled"])),
            body: "stuff".to_string(),
            else_body: None,
        },
        HelmExpr::Range {
            collection: Box::new(HelmExpr::value(&["items"])),
            body: "body".to_string(),
        },
        HelmExpr::With {
            context: Box::new(HelmExpr::value(&["ctx"])),
            body: "body".to_string(),
        },
        HelmExpr::Define {
            name: "name".to_string(),
            body: "body".to_string(),
        },
        HelmExpr::Pipe {
            expr: Box::new(HelmExpr::value(&["x"])),
            functions: vec!["quote".to_string()],
        },
    ];
    let before = samples.len();
    let mut ids: Vec<u8> = samples.iter().map(SynthesizerNode::variant_id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(
        ids.len(),
        before,
        "variant_id must be distinct for disjoint variants"
    );
}

// ─── SynthesizerNode laws ───────────────────────────────────────────

#[test]
fn law_determinism_holds_on_simple_nodes() {
    for n in [
        HelmExpr::value(&["replicaCount"]),
        HelmExpr::include("chart.fullname"),
        HelmExpr::chart("Name"),
        HelmExpr::image_ref(),
    ] {
        assert!(laws::is_deterministic(&n, 0));
        assert!(laws::is_deterministic(&n, 3));
    }
}

#[test]
fn law_determinism_holds_on_include_nindent() {
    let n = HelmExpr::include_nindent("chart.labels", 4);
    assert!(laws::is_deterministic(&n, 0));
    assert!(laws::is_deterministic(&n, 2));
}

#[test]
fn law_determinism_holds_on_interpolated() {
    let n = HelmExpr::Interpolated(vec![
        HelmExprPart::ValueRef(vec!["image".into(), "repository".into()]),
        HelmExprPart::Literal(":".into()),
        HelmExprPart::ValueRef(vec!["image".into(), "tag".into()]),
    ]);
    assert!(laws::is_deterministic(&n, 1));
}

#[test]
fn law_determinism_holds_on_if() {
    let n = HelmExpr::If {
        condition: Box::new(HelmExpr::value(&["enabled"])),
        body: "body content".to_string(),
        else_body: Some("else content".to_string()),
    };
    assert!(laws::is_deterministic(&n, 0));
    assert!(laws::is_deterministic(&n, 4));
}

#[test]
fn law_honors_indent_unit_on_value() {
    assert!(laws::honors_indent_unit(
        &HelmExpr::value(&["replicaCount"]),
        0
    ));
    assert!(laws::honors_indent_unit(
        &HelmExpr::value(&["replicaCount"]),
        2
    ));
}

#[test]
fn law_honors_indent_unit_on_include() {
    assert!(laws::honors_indent_unit(
        &HelmExpr::include("chart.fullname"),
        0
    ));
    assert!(laws::honors_indent_unit(
        &HelmExpr::include("chart.fullname"),
        5
    ));
}

#[test]
fn law_indent_monotone_len_on_value() {
    assert!(laws::indent_monotone_len(
        &HelmExpr::value(&["x"]),
        0
    ));
    assert!(laws::indent_monotone_len(
        &HelmExpr::value(&["x"]),
        3
    ));
}

#[test]
fn law_indent_monotone_len_on_chart() {
    assert!(laws::indent_monotone_len(&HelmExpr::chart("Name"), 0));
    assert!(laws::indent_monotone_len(&HelmExpr::chart("Name"), 7));
}

#[test]
fn law_variant_id_valid_on_all_sample_variants() {
    let samples = [
        HelmExpr::value(&["x"]),
        HelmExpr::include("t"),
        HelmExpr::include_nindent("t", 4),
        HelmExpr::chart("Name"),
        HelmExpr::image_ref(),
    ];
    for n in &samples {
        assert!(laws::variant_id_is_valid(n));
    }
}

// ─── NoRawAttestation ───────────────────────────────────────────────

#[test]
fn attestation_is_nonempty() {
    assert!(!<HelmExpr as NoRawAttestation>::attestation().is_empty());
}

#[test]
fn attestation_mentions_raw() {
    let s = <HelmExpr as NoRawAttestation>::attestation();
    assert!(
        s.to_lowercase().contains("raw"),
        "attestation must explain how no-raw is enforced — got: {s}"
    );
}

// ─── No-raw source invariant ────────────────────────────────────────

#[test]
fn no_raw_constructor_in_production_source() {
    // Scan src/ for `HelmExpr::Raw(...)` or `Self::Raw(...)` constructor
    // uses. Legitimate non-constructions (variant declaration, match arms,
    // #[allow(deprecated)]-pinned references, comments, attribute lines)
    // are exempted. HelmExpr has no Raw variant today — this test guards
    // against future reintroduction.
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();
    for path in walk_rust_files(&src_dir) {
        let content = std::fs::read_to_string(&path).expect("read src file");
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }
            // Variant declaration line.
            if line.contains("Raw(String)") {
                continue;
            }
            // Match arms (patterns, not constructions).
            if line.contains("=>") {
                continue;
            }
            // Attribute lines.
            if trimmed.starts_with("#[") {
                continue;
            }
            // Preceding #[allow(deprecated)] → intentional reference.
            let prev_allows = i > 0 && lines[i - 1].contains("#[allow(deprecated)]");
            if prev_allows {
                continue;
            }
            if line.contains("HelmExpr::Raw(") || line.contains("Self::Raw(") {
                violations.push(format!("{}:{}", path.display(), i + 1));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "HelmExpr::Raw construction in production source is forbidden \
         (use a typed variant). Violations: {violations:?}"
    );
}

fn walk_rust_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(root).expect("read src dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_rust_files(&path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}
