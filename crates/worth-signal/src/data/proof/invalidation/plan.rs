mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;

use super::super::locality::{PartitionScopeSet, TouchedScopeSummary};
use super::super::SummaryForm;
use super::frontier_admission::{
    FrontierEntryClassification, FrontierInclusionBasis, InvalidationSeedBatch,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FrontierWaveEntryPlan {
    pub(crate) node: NodeId,
    pub(crate) classification: FrontierEntryClassification,
    pub(crate) inclusion_basis: FrontierInclusionBasis,
    pub(crate) narrowed_scopes: PartitionScopeSet,
    pub(crate) source_seed_refs: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FrontierWavePlan {
    pub(crate) wave_index: u32,
    pub(crate) aspect: Aspect,
    pub(crate) entries: Vec<FrontierWaveEntryPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InvalidationPlanningEstimate {
    pub(crate) seed_count: u64,
    pub(crate) group_count: u64,
    pub(crate) direct_wave_count: u64,
    pub(crate) transitive_wave_count: u64,
    pub(crate) direct_dirty_count: u64,
    pub(crate) maybe_stale_count: u64,
    pub(crate) partition_scoped_checks: u64,
    pub(crate) partition_match_count: u64,
    pub(crate) detail_match_count: u64,
    pub(crate) cycle_check_candidate_count: u64,
}

impl InvalidationPlanningEstimate {
    pub const fn seed_count(&self) -> u64 {
        self.seed_count
    }

    pub const fn direct_candidate_count(&self) -> u64 {
        self.direct_dirty_count + self.maybe_stale_count
    }

    pub const fn partition_scoped_check_count(&self) -> u64 {
        self.partition_scoped_checks
    }
}

impl SummaryForm for InvalidationPlanningEstimate {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FrontierPlan {
    pub(crate) seed_batch: InvalidationSeedBatch,
    pub(crate) direct_waves: Vec<FrontierWavePlan>,
    pub(crate) touched_scope_summary: TouchedScopeSummary,
    pub(crate) predicted: InvalidationPlanningEstimate,
}

impl FrontierPlan {
    pub(crate) fn new(
        seed_batch: InvalidationSeedBatch,
        direct_waves: Vec<FrontierWavePlan>,
        touched_scope_summary: TouchedScopeSummary,
        predicted: InvalidationPlanningEstimate,
    ) -> Self {
        Self {
            seed_batch,
            direct_waves,
            touched_scope_summary,
            predicted,
        }
    }
}

impl SummaryForm for FrontierPlan {}
