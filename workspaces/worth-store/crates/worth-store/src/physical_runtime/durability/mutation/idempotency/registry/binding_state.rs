use crate::physical_runtime::{
    PhysicalDurabilityGroupMemberBinding, PhysicalMutationRequestFingerprint,
};

use super::super::{
    fate::PersistedPhysicalMutationFate, PhysicalMutationIdempotencyKey,
    PhysicalMutationIdempotencyKeyIdentity, PhysicalNamespaceDurableCheckpointGeneration,
};
use crate::physical_runtime::durability::mutation::PhysicalMutationIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalMutationBindingBasis {
    key: PhysicalMutationIdempotencyKey,
    fingerprint: PhysicalMutationRequestFingerprint,
    mutation: PhysicalMutationIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalMutationUnresolvedBindingObservation {
    key: PhysicalMutationIdempotencyKeyIdentity,
    fingerprint: PhysicalMutationRequestFingerprint,
    mutation: PhysicalMutationIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime::durability::mutation::idempotency) enum PhysicalMutationIdempotencyBindingState
{
    Unsealed(PhysicalMutationBindingBasis),
    GroupSealed {
        basis: PhysicalMutationBindingBasis,
        group: PhysicalDurabilityGroupMemberBinding,
    },
    RebuiltUnresolved {
        basis: PhysicalMutationBindingBasis,
        prior: RebuiltPhysicalMutationBindingState,
    },
    WalBound {
        basis: PhysicalMutationBindingBasis,
        persisted: super::super::PersistedPhysicalMutationAttemptBinding,
    },
    Terminal {
        basis: PhysicalMutationBindingBasis,
        fate: PersistedPhysicalMutationFate,
        last_compacted: Option<PhysicalNamespaceDurableCheckpointGeneration>,
    },
}

impl PhysicalMutationIdempotencyBindingState {
    /// Pending-unresolved capacity bounds obligations whose effect is not yet
    /// WAL-durable. A WAL-bound attempt's redo is already durable in retained
    /// WAL or the namespace-durable compaction, so only the total live-binding
    /// bound covers it. Live admission and fresh-process reopen share this one
    /// classification: reopen cannot refuse a registry the session held.
    pub(in crate::physical_runtime::durability::mutation::idempotency) const fn holds_pending_capacity(
        &self,
    ) -> bool {
        matches!(
            self,
            Self::Unsealed(_) | Self::GroupSealed { .. } | Self::RebuiltUnresolved { .. }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::durability::mutation::idempotency) enum RebuiltPhysicalMutationBindingState
{
    Unsealed,
    GroupSealed(PhysicalDurabilityGroupMemberBinding),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalMutationGroupSealingBinding {
    observation: PhysicalMutationUnresolvedBindingObservation,
    group: PhysicalDurabilityGroupMemberBinding,
}

impl PhysicalMutationUnresolvedBindingObservation {
    pub(in crate::physical_runtime) const fn new(
        key: PhysicalMutationIdempotencyKeyIdentity,
        fingerprint: PhysicalMutationRequestFingerprint,
        mutation: PhysicalMutationIdentity,
    ) -> Self {
        Self {
            key,
            fingerprint,
            mutation,
        }
    }

    pub(in crate::physical_runtime) const fn key(self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.key
    }

    pub(in crate::physical_runtime) const fn fingerprint(
        self,
    ) -> PhysicalMutationRequestFingerprint {
        self.fingerprint
    }

    pub(in crate::physical_runtime) const fn mutation(self) -> PhysicalMutationIdentity {
        self.mutation
    }
}

impl PhysicalMutationBindingBasis {
    pub(in crate::physical_runtime::durability::mutation::idempotency) fn new(
        key: PhysicalMutationIdempotencyKey,
        fingerprint: PhysicalMutationRequestFingerprint,
        mutation: PhysicalMutationIdentity,
    ) -> Self {
        Self {
            key,
            fingerprint,
            mutation,
        }
    }

    pub(in crate::physical_runtime) fn key(&self) -> &PhysicalMutationIdempotencyKey {
        &self.key
    }

    pub(in crate::physical_runtime) const fn fingerprint(
        &self,
    ) -> PhysicalMutationRequestFingerprint {
        self.fingerprint
    }

    pub(in crate::physical_runtime) const fn mutation(&self) -> PhysicalMutationIdentity {
        self.mutation
    }

    pub(super) fn observation(&self) -> PhysicalMutationUnresolvedBindingObservation {
        PhysicalMutationUnresolvedBindingObservation::new(
            self.key.identity(),
            self.fingerprint,
            self.mutation,
        )
    }
}

impl PhysicalMutationGroupSealingBinding {
    pub(in crate::physical_runtime) const fn new(
        observation: PhysicalMutationUnresolvedBindingObservation,
        group: PhysicalDurabilityGroupMemberBinding,
    ) -> Self {
        Self { observation, group }
    }

    pub(super) const fn observation(self) -> PhysicalMutationUnresolvedBindingObservation {
        self.observation
    }

    pub(super) const fn group(self) -> PhysicalDurabilityGroupMemberBinding {
        self.group
    }
}
