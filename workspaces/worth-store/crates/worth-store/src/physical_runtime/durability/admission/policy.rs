use std::num::{NonZeroU32, NonZeroU64};

use worth_proof::{ProofOutcome, TransitionOutcome};
use worth_signal::facade::TemporalDuration;
use worth_store_physical_backend::{
    BackendCapabilityKind, BackendTargetProfile, CapabilityEvidenceClass,
    PhysicalDurabilityAdmissionBasis, PhysicalDurabilityAdmissionIdentity,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::PhysicalWalPolicy;
use super::{
    PhysicalDurabilityPolicyDeferred, PhysicalDurabilityPolicyDenial,
    PhysicalDurabilityPolicyFailure, PhysicalDurabilityPolicyRebindRequired,
    PhysicalDurabilityPolicyStale,
};

mod identity;

use identity::policy_identity;
pub use identity::ConfiguredPhysicalDurabilityDeclaration;

pub type PhysicalDurabilityPolicyAdmissionOutcome = ProofOutcome<
    AdmittedPhysicalDurabilityPolicy,
    PhysicalDurabilityPolicyDenial,
    PhysicalDurabilityPolicyDeferred,
    PhysicalDurabilityPolicyStale,
    PhysicalDurabilityPolicyRebindRequired,
    PhysicalDurabilityPolicyFailure,
>;

macro_rules! nonzero_limit {
    ($name:ident, $inner:ty) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name($inner);

        impl $name {
            pub const fn new(value: $inner) -> Self {
                Self(value)
            }

            pub const fn get(self) -> $inner {
                self.0
            }
        }
    };
}

nonzero_limit!(GroupCommitLimit, NonZeroU32);
nonzero_limit!(IdempotencyRetentionGenerations, NonZeroU64);
nonzero_limit!(PendingUnresolvedMutationLimit, NonZeroU32);
nonzero_limit!(LiveIdempotencyBindingLimit, NonZeroU32);
nonzero_limit!(CheckpointMemoryLimit, NonZeroU64);
nonzero_limit!(RetainedWalTailLimit, NonZeroU64);

/// Maximum authoritative Signal-clock time a mutation may wait for grouping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroupCommitDelay(TemporalDuration);

impl GroupCommitDelay {
    pub fn new(milliseconds: NonZeroU64) -> Self {
        Self(
            TemporalDuration::temporal_duration(milliseconds.get())
                .expect("a nonzero group delay is a valid Signal duration"),
        )
    }

    pub const fn signal_duration(self) -> TemporalDuration {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalIdempotencyPolicy {
    retention: IdempotencyRetentionGenerations,
    pending_unresolved: PendingUnresolvedMutationLimit,
    live_bindings: LiveIdempotencyBindingLimit,
}

impl PhysicalIdempotencyPolicy {
    pub const fn new(
        retention: IdempotencyRetentionGenerations,
        pending_unresolved: PendingUnresolvedMutationLimit,
        live_bindings: LiveIdempotencyBindingLimit,
    ) -> Self {
        Self {
            retention,
            pending_unresolved,
            live_bindings,
        }
    }

    pub const fn retention(self) -> IdempotencyRetentionGenerations {
        self.retention
    }

    pub const fn pending_unresolved_limit(self) -> PendingUnresolvedMutationLimit {
        self.pending_unresolved
    }

    pub const fn live_binding_limit(self) -> LiveIdempotencyBindingLimit {
        self.live_bindings
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_recovery_binding(retention: NonZeroU64) -> Self {
        Self::new(
            IdempotencyRetentionGenerations::new(retention),
            PendingUnresolvedMutationLimit::new(NonZeroU32::MIN),
            LiveIdempotencyBindingLimit::new(NonZeroU32::MIN),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalCheckpointPolicy {
    memory: CheckpointMemoryLimit,
    retained_wal_tail: RetainedWalTailLimit,
}

impl PhysicalCheckpointPolicy {
    pub const fn fuzzy(
        memory: CheckpointMemoryLimit,
        retained_wal_tail: RetainedWalTailLimit,
    ) -> Self {
        Self {
            memory,
            retained_wal_tail,
        }
    }

    pub const fn memory_limit(self) -> CheckpointMemoryLimit {
        self.memory
    }

    pub const fn retained_wal_tail_limit(self) -> RetainedWalTailLimit {
        self.retained_wal_tail
    }
}

#[derive(Debug)]
pub struct PhysicalDurabilityDeclaration;

impl PhysicalDurabilityDeclaration {
    pub const fn builder() -> PhysicalDurabilityDeclarationBuilder<
        GroupMissing,
        WalMissing,
        IdempotencyMissing,
        CheckpointMissing,
    > {
        PhysicalDurabilityDeclarationBuilder {
            group: GroupMissing,
            wal: WalMissing,
            idempotency: IdempotencyMissing,
            checkpoint: CheckpointMissing,
        }
    }
}

#[doc(hidden)]
pub struct GroupMissing;
#[doc(hidden)]
pub struct WalMissing;
#[doc(hidden)]
pub struct IdempotencyMissing;
#[doc(hidden)]
pub struct CheckpointMissing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GroupPolicy {
    limit: GroupCommitLimit,
    delay: GroupCommitDelay,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroupConfigured(GroupPolicy);
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WalConfigured(PhysicalWalPolicy);
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdempotencyConfigured(PhysicalIdempotencyPolicy);
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckpointConfigured(PhysicalCheckpointPolicy);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalDurabilityDeclarationBuilder<Group, Wal, Idempotency, Checkpoint> {
    group: Group,
    wal: Wal,
    idempotency: Idempotency,
    checkpoint: Checkpoint,
}

impl<Wal, Idempotency, Checkpoint>
    PhysicalDurabilityDeclarationBuilder<GroupMissing, Wal, Idempotency, Checkpoint>
{
    pub fn group_commit(
        self,
        limit: GroupCommitLimit,
        delay: GroupCommitDelay,
    ) -> PhysicalDurabilityDeclarationBuilder<GroupConfigured, Wal, Idempotency, Checkpoint> {
        PhysicalDurabilityDeclarationBuilder {
            group: GroupConfigured(GroupPolicy { limit, delay }),
            wal: self.wal,
            idempotency: self.idempotency,
            checkpoint: self.checkpoint,
        }
    }
}

impl<Group, Idempotency, Checkpoint>
    PhysicalDurabilityDeclarationBuilder<Group, WalMissing, Idempotency, Checkpoint>
{
    pub fn wal(
        self,
        policy: PhysicalWalPolicy,
    ) -> PhysicalDurabilityDeclarationBuilder<Group, WalConfigured, Idempotency, Checkpoint> {
        PhysicalDurabilityDeclarationBuilder {
            group: self.group,
            wal: WalConfigured(policy),
            idempotency: self.idempotency,
            checkpoint: self.checkpoint,
        }
    }
}

impl<Group, Wal, Checkpoint>
    PhysicalDurabilityDeclarationBuilder<Group, Wal, IdempotencyMissing, Checkpoint>
{
    pub fn idempotency(
        self,
        policy: PhysicalIdempotencyPolicy,
    ) -> PhysicalDurabilityDeclarationBuilder<Group, Wal, IdempotencyConfigured, Checkpoint> {
        PhysicalDurabilityDeclarationBuilder {
            group: self.group,
            wal: self.wal,
            idempotency: IdempotencyConfigured(policy),
            checkpoint: self.checkpoint,
        }
    }
}

impl<Group, Wal, Idempotency>
    PhysicalDurabilityDeclarationBuilder<Group, Wal, Idempotency, CheckpointMissing>
{
    pub fn checkpoint(
        self,
        policy: PhysicalCheckpointPolicy,
    ) -> PhysicalDurabilityDeclarationBuilder<Group, Wal, Idempotency, CheckpointConfigured> {
        PhysicalDurabilityDeclarationBuilder {
            group: self.group,
            wal: self.wal,
            idempotency: self.idempotency,
            checkpoint: CheckpointConfigured(policy),
        }
    }
}

impl
    PhysicalDurabilityDeclarationBuilder<
        GroupConfigured,
        WalConfigured,
        IdempotencyConfigured,
        CheckpointConfigured,
    >
{
    pub fn admit(
        self,
        basis: PhysicalDurabilityAdmissionBasis,
    ) -> PhysicalDurabilityPolicyAdmissionOutcome {
        if !matches!(
            basis.target_profile(),
            BackendTargetProfile::PosixFileFsyncDirSync
                | BackendTargetProfile::WindowsFlushFileBuffers
        ) {
            return TransitionOutcome::denied(
                PhysicalDurabilityPolicyDenial::UnsupportedBackendProfile {
                    profile: basis.target_profile(),
                },
            )
            .into();
        }
        for (claim, capability) in [
            (basis.file_sync_claim(), BackendCapabilityKind::Fsync),
            (
                basis.directory_sync_claim(),
                BackendCapabilityKind::DirectorySync,
            ),
            (
                basis.durable_rename_claim(),
                BackendCapabilityKind::DurableRename,
            ),
        ] {
            if claim.kind() != capability
                || claim.profile() != basis.target_profile()
                || claim.evidence_class()
                    != CapabilityEvidenceClass::EstablishedByFilesystemAdmission
            {
                return TransitionOutcome::denied(
                    PhysicalDurabilityPolicyDenial::InvalidCapabilityBinding { capability },
                )
                .into();
            }
        }
        TransitionOutcome::success(AdmittedPhysicalDurabilityPolicy::new(
            basis,
            self.group.0,
            self.wal.0,
            self.idempotency.0,
            self.checkpoint.0,
        ))
        .into()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PhysicalDurabilityPolicyIdentity([u8; 32]);

impl PhysicalDurabilityPolicyIdentity {
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) const fn from_recovery_binding(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

pub struct AdmittedPhysicalDurabilityPolicy {
    identity: PhysicalDurabilityPolicyIdentity,
    basis: PhysicalDurabilityAdmissionIdentity,
    store: StableStoreIdentity,
    profile: BackendTargetProfile,
    group: GroupPolicy,
    wal: PhysicalWalPolicy,
    idempotency: PhysicalIdempotencyPolicy,
    checkpoint: PhysicalCheckpointPolicy,
}

impl AdmittedPhysicalDurabilityPolicy {
    fn new(
        basis: PhysicalDurabilityAdmissionBasis,
        group: GroupPolicy,
        wal: PhysicalWalPolicy,
        idempotency: PhysicalIdempotencyPolicy,
        checkpoint: PhysicalCheckpointPolicy,
    ) -> Self {
        let identity = policy_identity(basis.identity(), group, wal, idempotency, checkpoint);
        Self {
            identity,
            basis: basis.identity(),
            store: basis.store_identity(),
            profile: basis.target_profile(),
            group,
            wal,
            idempotency,
            checkpoint,
        }
    }

    pub const fn identity(&self) -> PhysicalDurabilityPolicyIdentity {
        self.identity
    }

    pub const fn admission_basis_identity(&self) -> PhysicalDurabilityAdmissionIdentity {
        self.basis
    }

    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }

    pub const fn profile(&self) -> BackendTargetProfile {
        self.profile
    }

    pub const fn group_commit_limit(&self) -> GroupCommitLimit {
        self.group.limit
    }

    pub const fn group_commit_delay(&self) -> GroupCommitDelay {
        self.group.delay
    }

    pub const fn wal_policy(&self) -> PhysicalWalPolicy {
        self.wal
    }

    pub const fn idempotency_policy(&self) -> PhysicalIdempotencyPolicy {
        self.idempotency
    }

    pub const fn checkpoint_policy(&self) -> PhysicalCheckpointPolicy {
        self.checkpoint
    }
}
