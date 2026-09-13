//! Read-only observation of the durability policy bound to a serving runtime.

use worth_store_physical_backend::{BackendTargetProfile, PhysicalDurabilityAdmissionIdentity};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::physical_runtime::RuntimeIdentity;

use super::super::{
    AdmittedPhysicalDurabilityPolicy, GroupCommitDelay, GroupCommitLimit, PhysicalCheckpointPolicy,
    PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy, PhysicalWalPolicy,
};

/// Read-only projection of the durability policy bound to one serving runtime.
///
/// The observation carries identity and admitted limits. It cannot reconstruct
/// the move-owned policy, platform basis, or runtime owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalDurabilityObservation {
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    policy: PhysicalDurabilityPolicyIdentity,
    admission_basis: PhysicalDurabilityAdmissionIdentity,
    profile: BackendTargetProfile,
    group_commit_limit: GroupCommitLimit,
    group_commit_delay: GroupCommitDelay,
    wal: PhysicalWalPolicy,
    idempotency: PhysicalIdempotencyPolicy,
    checkpoint: PhysicalCheckpointPolicy,
    reopen: Option<PhysicalDurabilityReopenObservation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalDurabilityReopenObservation {
    checkpoint_artifact_bytes: u64,
    checkpoint_bytes_read: u64,
    dirty_body_bytes_skipped: u64,
    binding_records_read: u64,
    checkpoint_integrity_admissions: u64,
    wal_members_read: u64,
}

impl PhysicalDurabilityObservation {
    pub(in crate::physical_runtime) fn new(
        runtime: RuntimeIdentity,
        policy: &AdmittedPhysicalDurabilityPolicy,
    ) -> Self {
        Self {
            store: policy.store_identity(),
            runtime,
            policy: policy.identity(),
            admission_basis: policy.admission_basis_identity(),
            profile: policy.profile(),
            group_commit_limit: policy.group_commit_limit(),
            group_commit_delay: policy.group_commit_delay(),
            wal: policy.wal_policy(),
            idempotency: policy.idempotency_policy(),
            checkpoint: policy.checkpoint_policy(),
            reopen: None,
        }
    }

    pub const fn store_identity(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn runtime_identity(self) -> RuntimeIdentity {
        self.runtime
    }

    pub const fn policy_identity(self) -> PhysicalDurabilityPolicyIdentity {
        self.policy
    }

    pub const fn admission_basis_identity(self) -> PhysicalDurabilityAdmissionIdentity {
        self.admission_basis
    }

    pub const fn profile(self) -> BackendTargetProfile {
        self.profile
    }

    pub const fn group_commit_limit(self) -> GroupCommitLimit {
        self.group_commit_limit
    }

    pub const fn group_commit_delay(self) -> GroupCommitDelay {
        self.group_commit_delay
    }

    pub const fn wal_policy(self) -> PhysicalWalPolicy {
        self.wal
    }

    pub const fn idempotency_policy(self) -> PhysicalIdempotencyPolicy {
        self.idempotency
    }

    pub const fn checkpoint_policy(self) -> PhysicalCheckpointPolicy {
        self.checkpoint
    }

    pub const fn reopen(self) -> Option<PhysicalDurabilityReopenObservation> {
        self.reopen
    }

    pub(in crate::physical_runtime) const fn with_reopen(
        mut self,
        reopen: PhysicalDurabilityReopenObservation,
    ) -> Self {
        self.reopen = Some(reopen);
        self
    }
}

impl PhysicalDurabilityReopenObservation {
    pub(in crate::physical_runtime) const fn new(
        checkpoint_artifact_bytes: u64,
        checkpoint_bytes_read: u64,
        dirty_body_bytes_skipped: u64,
        binding_records_read: u64,
        checkpoint_integrity_admissions: u64,
        wal_members_read: u64,
    ) -> Self {
        Self {
            checkpoint_artifact_bytes,
            checkpoint_bytes_read,
            dirty_body_bytes_skipped,
            binding_records_read,
            checkpoint_integrity_admissions,
            wal_members_read,
        }
    }

    pub const fn checkpoint_artifact_bytes(self) -> u64 {
        self.checkpoint_artifact_bytes
    }

    pub const fn checkpoint_bytes_read(self) -> u64 {
        self.checkpoint_bytes_read
    }

    pub const fn dirty_body_bytes_skipped(self) -> u64 {
        self.dirty_body_bytes_skipped
    }

    pub const fn binding_records_read(self) -> u64 {
        self.binding_records_read
    }

    pub const fn checkpoint_integrity_admissions(self) -> u64 {
        self.checkpoint_integrity_admissions
    }

    pub const fn wal_members_read(self) -> u64 {
        self.wal_members_read
    }
}
