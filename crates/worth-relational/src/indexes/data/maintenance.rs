use serde::{Deserialize, Serialize};

use super::{DerivedIndexGeneration, DerivedIndexId};

/// Explicit limits for patch-local work and optional cold reconstruction.
/// A zero cold limit prohibits reconstruction, including a missing generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexMaintenanceBudget {
    pub maximum_work_units: usize,
    pub maximum_cold_record_slots: usize,
    pub maximum_derived_rows: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexMaintenanceWork {
    pub work_units: usize,
    pub patch_records: usize,
    pub record_reads: usize,
    pub adjacency_work_units: usize,
    pub entry_edits: usize,
    pub seek_comparisons: usize,
    /// Conservative handle-copy reservation, not a measured allocation count.
    pub path_copy_units_reserved: usize,
    pub cold_record_slots: usize,
    pub derived_rows: usize,
    pub reused_generations: usize,
    /// Catalog insertions prepared for the performed-commit finalizer.
    #[serde(default)]
    pub generation_publications_reserved: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivedIndexMaintenanceDenialKind {
    Basis(crate::branch::RelationalBranchBasisDenial),
    SnapshotUnavailable,
    CommitMismatch,
    BeforeRootMismatch,
    IndexUnavailable(DerivedIndexId),
    GenerationKindMismatch(DerivedIndexId),
    PriorEntryMismatch,
    ColdReconstructionRequired,
    WorkBudgetExceeded,
    ForeignCandidate,
    CandidateLifetimeExpired { maximum_lifetime_millis: u64 },
    CandidateUnavailable,
    CandidateIndexesAlreadyPrepared,
    GenerationIdentityExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexMaintenanceDenial {
    pub kind: DerivedIndexMaintenanceDenialKind,
    pub work: DerivedIndexMaintenanceWork,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexMaintenanceOutcome {
    pub generations: Vec<DerivedIndexGeneration>,
    pub work: DerivedIndexMaintenanceWork,
}
