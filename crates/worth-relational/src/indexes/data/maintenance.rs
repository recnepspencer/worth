use serde::{Deserialize, Serialize};

use super::{DerivedIndexGeneration, DerivedIndexId};

/// Limits for change maintenance or an explicitly entered reconstruction.
/// A cold allowance grants no reconstruction authority: ordinary maintenance
/// refuses missing predecessor state even when this allowance is positive or absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexMaintenanceBudget {
    pub maximum_work_units: Option<usize>,
    pub maximum_cold_record_slots: Option<usize>,
    pub maximum_derived_rows: Option<usize>,
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
    /// Fork generations published from the authoring branch's generation at
    /// the same exact root, without cold projection.
    #[serde(default)]
    pub seeded_generations: usize,
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
    Execution(crate::execution::RelationalExecutionDenialCause),
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

/// Refusal before an exact selected-generation lookup performs its next read.
/// The caller's admission owns the resource category and cumulative budget.
#[derive(Debug)]
pub enum SelectedIndexGenerationAdmissionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

#[derive(Debug)]
pub enum DerivedIndexMaintenanceAdmissionStop<Stop> {
    Native(DerivedIndexMaintenanceDenial),
    Admission(Stop),
}

impl From<crate::storage::substrate::maintenance::MaintenanceRefusal>
    for DerivedIndexMaintenanceDenial
{
    fn from(cause: crate::storage::substrate::maintenance::MaintenanceRefusal) -> Self {
        use crate::storage::substrate::maintenance::MaintenanceRefusal as Refusal;
        Self {
            kind: match cause {
                Refusal::Execution(cause) => DerivedIndexMaintenanceDenialKind::Execution(cause),
                Refusal::MissingPredecessor | Refusal::UnavailableChange => {
                    DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
                }
            },
            work: Default::default(),
        }
    }
}

impl From<crate::storage::substrate::maintenance::MaintenanceRefusal>
    for DerivedIndexMaintenanceDenialKind
{
    fn from(cause: crate::storage::substrate::maintenance::MaintenanceRefusal) -> Self {
        use crate::storage::substrate::maintenance::MaintenanceRefusal as Refusal;
        match cause {
            Refusal::Execution(cause) => Self::Execution(cause),
            Refusal::MissingPredecessor | Refusal::UnavailableChange => {
                Self::ColdReconstructionRequired
            }
        }
    }
}
