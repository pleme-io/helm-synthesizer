//! Conformance to [`synthesizer_core`] traits.
//!
//! Wave 2 of the compound-knowledge refactor: purely additive. No behavior
//! change to helm-synthesizer's existing APIs — this module only adds trait
//! impls that downstream generic code can consume.
//!
//! Note: `HelmExpr`'s inherent `emit` takes no indent parameter because Helm
//! Go template expressions are typically inline within YAML. The trait impl
//! prepends `indent_unit.repeat(indent)` to the first line of the inherent
//! output so the `honors_indent_unit` and `indent_monotone_len` laws hold
//! non-trivially.

use crate::types::HelmExpr;
use synthesizer_core::{NoRawAttestation, SynthesizerNode};

impl SynthesizerNode for HelmExpr {
    fn emit(&self, indent: usize) -> String {
        // Inherent `HelmExpr::emit(&self) -> String` takes no indent — it
        // produces inline Go template expressions. The trait contract says
        // `emit(indent)` should honor indent; we prepend indent_unit*indent
        // to the first line. Single-arg inherent is unambiguous in UFCS
        // because the trait method has two args.
        let body = HelmExpr::emit(self);
        let pad = Self::indent_unit().repeat(indent);
        format!("{pad}{body}")
    }

    fn indent_unit() -> &'static str {
        "  "
    }

    fn variant_id(&self) -> u8 {
        match self {
            Self::Value(_) => 0,
            Self::Include { .. } => 1,
            Self::IncludeNindent { .. } => 2,
            Self::ChartField(_) => 3,
            Self::Interpolated(_) => 4,
            Self::If { .. } => 5,
            Self::Range { .. } => 6,
            Self::With { .. } => 7,
            Self::Define { .. } => 8,
            Self::Pipe { .. } => 9,
        }
    }
}

impl NoRawAttestation for HelmExpr {
    fn attestation() -> &'static str {
        "HelmExpr has no Raw variant and no escape hatch. Every Helm Go \
         template expression is one of 10 typed variants (Value, Include, \
         IncludeNindent, ChartField, Interpolated, If, Range, With, Define, \
         Pipe). Arbitrary string template bodies inside If/Range/With/Define \
         carry the pre-rendered inner body as `String`, but that inner body \
         is constructed from composed HelmExpr emissions, not raw user \
         input, by upstream callers. tests/synthesizer_core_conformance.rs \
         ::no_raw_constructor_in_production_source scans src/ for Raw \
         constructor patterns; any accidental reintroduction fails CI."
    }
}
