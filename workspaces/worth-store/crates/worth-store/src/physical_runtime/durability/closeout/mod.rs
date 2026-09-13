mod backend_evidence;
mod checkpoint_basis;
mod handoff;
mod operation_fates;
mod recovery_allocation;
mod residue;
mod root_basis;
mod wal_tail;

pub use backend_evidence::PhysicalBackendDurabilityCloseoutEvidence;
pub use checkpoint_basis::PhysicalRecoveryCheckpointBasis;
pub use handoff::{
    PhysicalDurabilityCloseoutDenial, PhysicalDurabilityCloseoutOutcome,
    PhysicalDurabilityRecoveryHandoff,
};
pub(in crate::physical_runtime) use operation_fates::PhysicalIdempotencyCloseoutDenial;
pub use operation_fates::{
    PhysicalRecoveryAttemptBindingFact, PhysicalRecoveryCompletedMutationFact,
    PhysicalRecoveryOperationFact, PhysicalRecoveryOperationFate,
    PhysicalRecoveryOperationFateCounts, PhysicalRecoveryOperationFates,
    PhysicalRecoveryWalAttemptBinding,
};
pub use recovery_allocation::PhysicalRecoveryAllocationAdmission;
pub use residue::PhysicalArtifactResidueClassification;
pub use root_basis::{PhysicalRecoveryRootBasis, PhysicalRootNamespaceDurabilityEvidence};
pub use wal_tail::{PhysicalRecoveryWalSegment, PhysicalRecoveryWalTail};
