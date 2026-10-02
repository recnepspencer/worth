use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::BlobManifestResidueCleanup;
use worth_store_wal::WalLsnRange;

use crate::physical_runtime::{PhysicalMutationIdentity, PhysicalMutationRequestFingerprint};

mod accessors;
mod checkpoint_basis;
mod evidence;
mod evidence_merge;
#[cfg(test)]
use evidence_merge::conflicting_terminal_fates;
mod failure;
mod manifest_cleanup;
#[cfg(test)]
mod merge_tests;
mod operations;
mod retirement_obligation;
mod sampling;
mod storage_allocation;
mod tier_epoch;
mod wal_frame_input;
mod wal_frame_view;
mod wal_payload;
pub use sampling::allocation::StoreRecoveryBindingSampleAllocationDenial;
pub use wal_frame_view::IntegrityAdmittedRecoveryWalFrameView;

pub(in crate::physical_runtime) use checkpoint_basis::{
    checkpoint_binding_decode_peak, decode_checkpoint_evidence,
};
pub use checkpoint_basis::{
    StoreRecoveryCheckpointBindingAllocationDenial, StoreRecoveryCheckpointBindingBasis,
    StoreRecoveryCheckpointBindingRebuilder,
};
pub use failure::StoreRecoveryBindingSampleFailure;
use failure::{empty_failure, sample_failure_from_evidence};
pub use retirement_obligation::{StoreRecoveryRetiredArtifact, StoreRecoveryRetirementObligation};
pub use tier_epoch::StoreTierEpochActivationObservation;
pub(super) use wal_frame_input::sample_binding;

#[derive(Debug)]
pub struct StoreRecoveryBindingFreshnessSample {
    store: StableStoreIdentity,
    selected_checkpoint_generation: u64,
    sealed_basis_identity: [u8; 32],
    policy_identity: [u8; 32],
    operations: Vec<StoreRecoveryOperationEvidence>,
    wal_members: Vec<StoreRecoveryWalMember>,
    retirements: Vec<StoreRecoveryRetirementObligation>,
    extent_copy_frames: Vec<(WalLsnRange, Vec<u8>)>,
    blob_manifest_residue_cleanups: Vec<(WalLsnRange, BlobManifestResidueCleanup, bool)>,
    tier_epoch_activation: Option<StoreTierEpochActivationObservation>,
    manifest_cleanup_sampling_peak_bytes: u64,
    backing: sampling::allocation::SamplingBacking,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRecoveryOperationEvidence {
    idempotency_identity: [u8; 32],
    mutation: PhysicalMutationIdentity,
    request_fingerprint: PhysicalMutationRequestFingerprint,
    lease_issuance_generation: u64,
    lease_expiry_generation: u64,
    freshness: StoreRecoveryBindingFreshness,
    fate: StoreRecoveryOperationFate,
    attempt_binding_identity: Option<[u8; 32]>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoreRecoveryWalMember {
    lsn_range: WalLsnRange,
    operation_identity: [u8; 32],
    group_identity: [u8; 32],
    group_member_identity: [u8; 32],
    group_member_ordinal: u32,
    group_member_count: u32,
    group_membership_digest: [u8; 32],
    canonical_redo: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreRecoveryBindingFreshness {
    Retained,
    ExpiredAtSelectedCheckpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreRecoveryOperationFate {
    AcknowledgedDurable,
    DurableUnacknowledged,
    ProvenNoEffect,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreRecoveryBindingSampleDenial {
    FreshnessMediaMismatch,
    ForeignCheckpoint,
    MissingCheckpointSecurityBinding,
    InvalidCheckpointSecurityBinding,
    InvalidCheckpointBinding,
    InvalidWalMember,
    ConflictingOperationEvidence,
    OperationBindingLimit,
    RedoByteLimit,
    RecoveryMemoryLimit,
}

/// Which WAL members the checkpoint already covers are sampled.
///
/// Planning redoes only the tail after the checkpoint cutoff; cleanup proves the
/// covered members it is about to delete are terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::recovery_freshness) enum CheckpointCoveredMembers {
    Skip,
    Sample,
}

fn checkpoint_evidence(
    record: crate::physical_runtime::durability::DecodedPhysicalMutationBindingRecord,
    selected_generation: u64,
) -> StoreRecoveryOperationEvidence {
    evidence::checkpoint_evidence(record, selected_generation)
}
