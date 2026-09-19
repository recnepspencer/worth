use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

mod retained_charge;

use crate::data::core_profile::StableHashValue;
use crate::data::output::{
    ArtifactContinuityToken, ChangedRegion, MemoizedResultOrigin, OutputChange, OutputIdentity,
};
use crate::data::reuse::{ReuseBasis, ReuseBoundaryContext, ReuseOrigin};

use super::authority::ArtifactTransitionKey;
use super::records::HistoricalArtifactRecord;
use super::summary::TraceSummary;
use super::tiers::RuntimeArtifactState;
use super::writes::ColdArtifactRecord;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticArtifactParity {
    output_hash: StableHashValue,
    output_identity: Option<OutputIdentity>,
    continuity_token: Option<ArtifactContinuityToken>,
    output_change: OutputChange,
    recomputed: bool,
    dependency_count: u32,
    meaningful_input_changes: u32,
    changed_partition_count: u32,
    propagation_suppressed: bool,
    changed_regions: Vec<ChangedRegion>,
    keyed_family: Option<String>,
    keyed_key: Option<String>,
    memoized_origin: MemoizedResultOrigin,
    reuse_basis: ReuseBasis,
    reuse_origin: ReuseOrigin,
    reuse_boundary_context: Option<ReuseBoundaryContext>,
    labels: Vec<String>,
    artifact_transition_key: ArtifactTransitionKey,
}

#[cfg_attr(not(test), allow(dead_code))]
impl SemanticArtifactParity {
    pub fn from_historical_artifact_record(record: &HistoricalArtifactRecord) -> Self {
        Self::from_runtime_and_cold(&record.runtime, record.retained.as_ref())
    }

    pub fn from_trace_summary(summary: &TraceSummary) -> Self {
        let mut changed_regions = summary.changed_regions.clone();
        changed_regions.sort();
        Self {
            output_hash: summary.output_hash,
            output_identity: summary.output_identity.clone(),
            continuity_token: summary.continuity_token.clone(),
            output_change: summary.output_change,
            recomputed: summary.recomputed,
            dependency_count: summary.dependency_count,
            meaningful_input_changes: summary.meaningful_input_changes,
            changed_partition_count: summary.changed_partition_count,
            propagation_suppressed: summary.propagation_suppressed,
            changed_regions,
            keyed_family: summary.keyed_family.clone(),
            keyed_key: summary.keyed_key.clone(),
            memoized_origin: summary.memoized_origin,
            reuse_basis: summary.reuse_basis.clone(),
            reuse_origin: summary.reuse_origin,
            reuse_boundary_context: summary.reuse_boundary_context.clone(),
            labels: summary.labels.clone(),
            artifact_transition_key: ArtifactTransitionKey::new(summary.lineage_artifact_id),
        }
    }

    fn from_runtime_and_cold(
        runtime: &RuntimeArtifactState,
        retained: Option<&ColdArtifactRecord>,
    ) -> Self {
        let mut changed_regions = retained
            .map(|artifact| artifact.changed_regions.as_slice().to_vec())
            .unwrap_or_else(|| {
                super::summary::scopes_to_regions_from_slice(runtime.changed_scopes().as_slice())
            });
        changed_regions.sort();
        Self {
            output_hash: runtime.output_hash(),
            output_identity: runtime.output_identity().cloned(),
            continuity_token: runtime.continuity_token_authority().clone_inner(),
            output_change: runtime.output_change(),
            recomputed: runtime.recomputed(),
            dependency_count: runtime.dependency_count(),
            meaningful_input_changes: runtime.meaningful_input_changes(),
            changed_partition_count: runtime.changed_partition_count(),
            propagation_suppressed: runtime.propagation_suppressed(),
            changed_regions,
            keyed_family: retained.and_then(|artifact| artifact.keyed_family.clone()),
            keyed_key: retained.and_then(|artifact| artifact.keyed_key.clone()),
            memoized_origin: runtime.memoized_origin(),
            reuse_basis: runtime.reuse_basis().clone_inner(),
            reuse_origin: runtime.reuse_origin(),
            reuse_boundary_context: retained
                .and_then(|artifact| artifact.reuse_boundary_context.clone()),
            labels: retained
                .map(|artifact| artifact.labels.clone())
                .unwrap_or_default(),
            artifact_transition_key: runtime.lineage_artifact_id(),
        }
    }
}

/// Opaque structured causality payload for host-provided provenance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CausalityMetadata {
    /// Stable kind label for the payload producer.
    pub kind: String,
    /// Opaque string fields surfaced in explanations and debug output.
    #[serde(default)]
    pub fields: BTreeMap<String, String>,
}
