mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;

use super::super::locality::{PartitionScopeSet, TouchedScopeSummary};
use super::super::SummaryForm;
use super::frontier_admission::{FrontierEntryClassification, FrontierInclusionBasis};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FrontierWaveEntrySummary {
    pub(crate) node: NodeId,
    pub(crate) classification: FrontierEntryClassification,
    pub(crate) inclusion_basis: FrontierInclusionBasis,
    pub(crate) narrowed_scopes: PartitionScopeSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FrontierWaveSummary {
    pub(crate) wave_index: u32,
    pub(crate) aspect: Aspect,
    pub(crate) entries: Vec<FrontierWaveEntrySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TransitiveFrontierWaveSummary {
    pub(crate) wave_index: u32,
    pub(crate) entries: Vec<TransitiveFrontierEntrySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TransitiveFrontierEntrySummary {
    pub(crate) node: NodeId,
    pub(crate) classification: FrontierEntryClassification,
    pub(crate) inclusion_basis: FrontierInclusionBasis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) struct FrontierDiagnosticsProjection {
    pub(crate) frontier_seed_count: u64,
    pub(crate) frontier_group_count: u64,
    pub(crate) frontier_direct_wave_count: u64,
    pub(crate) frontier_transitive_wave_count: u64,
    pub(crate) frontier_partition_scoped_check_count: u64,
    pub(crate) frontier_direct_dirty_count: u64,
    pub(crate) frontier_maybe_stale_count: u64,
    pub(crate) frontier_partition_match_count: u64,
    pub(crate) frontier_detail_match_count: u64,
    pub(crate) frontier_cycle_check_candidate_count: u64,
    pub(crate) frontier_cycle_check_visited_count: u64,
    pub(crate) frontier_trace_retained_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FrontierDiagnosticsSidecar {
    pub(crate) seed_count: u64,
    pub(crate) direct_waves: Vec<FrontierWaveSummary>,
    pub(crate) transitive_waves: Vec<TransitiveFrontierWaveSummary>,
    pub(crate) touched_scope_summary: TouchedScopeSummary,
    pub(crate) counters: FrontierDiagnosticsProjection,
}

impl FrontierDiagnosticsSidecar {
    pub(crate) fn new(
        seed_count: u64,
        direct_waves: Vec<FrontierWaveSummary>,
        transitive_waves: Vec<TransitiveFrontierWaveSummary>,
        touched_scope_summary: TouchedScopeSummary,
        counters: FrontierDiagnosticsProjection,
    ) -> Self {
        Self {
            seed_count,
            direct_waves,
            transitive_waves,
            touched_scope_summary,
            counters,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidationTraceRecord {
    pub node: NodeId,
    pub aspect: Aspect,
    pub wave_index: u32,
    pub classification: FrontierEntryClassification,
    pub inclusion_basis: FrontierInclusionBasis,
}

impl InvalidationTraceRecord {
    pub fn new(
        node: NodeId,
        aspect: Aspect,
        wave_index: u32,
        classification: FrontierEntryClassification,
        inclusion_basis: FrontierInclusionBasis,
    ) -> Self {
        Self {
            node,
            aspect,
            wave_index,
            classification,
            inclusion_basis,
        }
    }
}

impl SummaryForm for FrontierDiagnosticsSidecar {}
