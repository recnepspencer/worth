mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::handle::NodeId;
use crate::data::node::{ContextRequirement, NodeState};
use crate::data::output::OutputChange;
use crate::data::reuse::ReuseBasis;
use crate::diagnostics::policy::{DiagnosticsAvailability, RetentionBudget};
use crate::diagnostics::profile::DiagnosticsTier;
use crate::logic::explain::{CausalDisposition, NodeExplanation, UpstreamCause};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplanationSummary {
    pub profile: DiagnosticsTier,
    pub node: NodeId,
    pub materialization_mode: DiagnosticsAvailability,
    pub state: NodeState,
    pub dirty_aspect_count: u32,
    pub upstream_count: u32,
    pub changed_upstream_count: u32,
    pub skipped_upstream_count: u32,
    pub condition_deferred_count: u32,
    pub clean_upstream_count: u32,
    pub missing_snapshot_count: u32,
    pub dependency_removed_count: u32,
    pub conservative_cause_count: u32,
    pub direct_scope_count: u32,
    pub translated_scope_count: u32,
    pub discarded_scope_count: u32,
    pub insufficient_scope_count: u32,
    pub rewired_dependency_count: u32,
    pub direct_cause_kinds: Vec<crate::logic::explain::CausalLinkKind>,
    pub scope_provenance_kinds: Vec<String>,
    pub cause_note_samples: Vec<String>,
    pub triage_classes: Vec<String>,
    pub propagation_suppressed: bool,
    pub contract_reads_mask: u128,
    pub contract_produces_mask: u128,
    pub contract_partition_scope_count: u32,
    pub required_context: ContextRequirement,
    pub execution_record_id: Option<u64>,
    pub semantic_segment_id: Option<u64>,
    pub output_change: Option<OutputChange>,
    pub memoized_origin: Option<crate::data::output::MemoizedResultOrigin>,
    pub reuse_basis: Option<ReuseBasis>,
    pub reuse_origin: Option<crate::data::reuse::ReuseOrigin>,
    pub reuse_certification_proof_count: u32,
    pub changed_region_count: u32,
    pub causality_kind: Option<String>,
}

impl ExplanationSummary {
    pub fn from_explanation(explanation: &NodeExplanation, profile: DiagnosticsTier) -> Self {
        let mut changed_upstream_count = 0_u32;
        let mut skipped_upstream_count = 0_u32;
        let mut condition_deferred_count = 0_u32;
        let mut clean_upstream_count = 0_u32;
        let mut missing_snapshot_count = 0_u32;
        let mut dependency_removed_count = 0_u32;
        let mut conservative_cause_count = 0_u32;
        let mut direct_scope_count = 0_u32;
        let mut translated_scope_count = 0_u32;
        let mut discarded_scope_count = 0_u32;
        let mut insufficient_scope_count = 0_u32;
        let mut direct_cause_kinds = Vec::new();
        let mut scope_provenance_kinds = Vec::new();
        let mut cause_note_samples = Vec::new();
        let mut triage_classes = Vec::new();

        for cause in &explanation.upstream {
            match cause {
                UpstreamCause::Changed { .. } => changed_upstream_count += 1,
                UpstreamCause::SkippedByComparator { .. } => skipped_upstream_count += 1,
                UpstreamCause::ConditionDeferred { .. } => condition_deferred_count += 1,
                UpstreamCause::Clean { .. } => clean_upstream_count += 1,
                UpstreamCause::MissingSnapshot { .. } => missing_snapshot_count += 1,
                UpstreamCause::DependencyRemoved { .. } => dependency_removed_count += 1,
            }
        }
        let detail_limit = RetentionBudget::for_tier(profile).detail_limit.get();
        for link in &explanation.causal_links {
            if matches!(link.disposition, CausalDisposition::Conservative) {
                conservative_cause_count += 1;
            }
            match link.scope.kind {
                crate::logic::explain::ScopeProvenanceKind::Direct => direct_scope_count += 1,
                crate::logic::explain::ScopeProvenanceKind::Translated => {
                    translated_scope_count += 1
                }
                crate::logic::explain::ScopeProvenanceKind::Discarded => discarded_scope_count += 1,
                crate::logic::explain::ScopeProvenanceKind::InsufficientEvidence => {
                    insufficient_scope_count += 1
                }
                crate::logic::explain::ScopeProvenanceKind::None => {}
            }
            if direct_cause_kinds.len() < detail_limit {
                direct_cause_kinds.push(link.kind.clone());
            }
            if !matches!(
                link.scope.kind,
                crate::logic::explain::ScopeProvenanceKind::None
            ) && scope_provenance_kinds.len() < detail_limit
            {
                scope_provenance_kinds.push(format!("{:?}", link.scope.kind));
            }
            if let Some(note) = &link.note {
                if cause_note_samples.len() < detail_limit {
                    cause_note_samples.push(note.clone());
                }
            }
            push_triage_class(
                &mut triage_classes,
                triage_class_for_link(link, explanation.rewiring.is_some()),
            );
        }
        if explanation.rewiring.is_some() {
            push_triage_class(&mut triage_classes, Some("rewiring".to_string()));
        }

        Self {
            profile,
            node: explanation.node,
            materialization_mode: explanation.materialization_mode,
            state: explanation.state,
            dirty_aspect_count: explanation.dirty_aspects.bits().count_ones(),
            upstream_count: explanation.upstream.len() as u32,
            changed_upstream_count,
            skipped_upstream_count,
            condition_deferred_count,
            clean_upstream_count,
            missing_snapshot_count,
            dependency_removed_count,
            conservative_cause_count,
            direct_scope_count,
            translated_scope_count,
            discarded_scope_count,
            insufficient_scope_count,
            rewired_dependency_count: explanation
                .rewiring
                .as_ref()
                .map(|rewiring| (rewiring.added.len() + rewiring.removed.len()) as u32)
                .unwrap_or(0),
            direct_cause_kinds,
            scope_provenance_kinds,
            cause_note_samples,
            triage_classes,
            propagation_suppressed: explanation.propagation_suppressed,
            contract_reads_mask: explanation.contract_reads.bits() as u128,
            contract_produces_mask: explanation.contract_produces.bits() as u128,
            contract_partition_scope_count: explanation
                .contract_partition_scope
                .as_ref()
                .map(|scopes| scopes.len() as u32)
                .unwrap_or(0),
            required_context: explanation.required_context,
            execution_record_id: explanation.execution_record_id,
            semantic_segment_id: explanation.semantic_segment_id,
            output_change: explanation.output_change,
            memoized_origin: explanation.memoized_origin,
            reuse_basis: explanation.reuse_basis.clone(),
            reuse_origin: explanation.reuse_origin,
            reuse_certification_proof_count: explanation
                .reuse_certification
                .as_ref()
                .map(|record| record.proofs.len() as u32)
                .unwrap_or(0),
            changed_region_count: explanation.changed_regions.len() as u32,
            causality_kind: explanation.causality.as_ref().map(|c| c.kind.clone()),
        }
    }
}

impl NodeExplanation {
    pub fn diagnostics_summary(&self, profile: DiagnosticsTier) -> ExplanationSummary {
        ExplanationSummary::from_explanation(self, profile)
    }
}

fn triage_class_for_link(
    link: &crate::logic::explain::CausalLink,
    rewired: bool,
) -> Option<String> {
    if rewired || matches!(link.disposition, CausalDisposition::Topology) {
        return Some("rewiring".to_string());
    }
    if matches!(
        link.scope.kind,
        crate::logic::explain::ScopeProvenanceKind::Direct
            | crate::logic::explain::ScopeProvenanceKind::Translated
            | crate::logic::explain::ScopeProvenanceKind::Discarded
            | crate::logic::explain::ScopeProvenanceKind::InsufficientEvidence
    ) {
        return Some("locality".to_string());
    }
    if matches!(link.disposition, CausalDisposition::Conservative)
        || matches!(
            link.kind,
            crate::logic::explain::CausalLinkKind::SkippedByComparator
                | crate::logic::explain::CausalLinkKind::ConditionDeferred { .. }
        )
    {
        return Some("validation".to_string());
    }
    None
}

fn push_triage_class(target: &mut Vec<String>, value: Option<String>) {
    let Some(value) = value else {
        return;
    };
    if !target.contains(&value) {
        target.push(value);
    }
}
