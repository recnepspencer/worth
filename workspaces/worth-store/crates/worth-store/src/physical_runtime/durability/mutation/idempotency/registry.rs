use std::collections::BTreeMap;

mod admission;
mod binding_state;
mod closeout;

pub(super) use binding_state::{
    PhysicalMutationBindingBasis, PhysicalMutationIdempotencyBindingState,
    RebuiltPhysicalMutationBindingState,
};
pub(in crate::physical_runtime) use binding_state::{
    PhysicalMutationGroupSealingBinding, PhysicalMutationUnresolvedBindingObservation,
};

use crate::physical_runtime::{
    CompletedPhysicalMutationFact, IndeterminatePhysicalMutation, PendingUnresolvedMutationLimit,
    PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy,
    PhysicalMutationProvenNoEffectCause, ProvenNoEffectPhysicalMutation, RuntimeIdentity,
};
use std::sync::Arc;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::super::{PhysicalMutationIdentity, PhysicalMutationRequestFingerprint};
use super::{
    attempt_binding::UnallocatedPhysicalMutationAttemptBinding,
    fate::PersistedPhysicalMutationFate,
    lease::{PhysicalMutationLeaseIssuanceFailure, PhysicalNamespaceDurableCheckpointGeneration},
    PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyKeyIdentity,
    PhysicalMutationIdempotencyLease, PhysicalMutationIdempotencyMaterial,
};

pub(in crate::physical_runtime::durability) struct PhysicalMutationIdempotencyRegistry {
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    policy: PhysicalDurabilityPolicyIdentity,
    retention: crate::physical_runtime::IdempotencyRetentionGenerations,
    pending_limit: PendingUnresolvedMutationLimit,
    live_limit: crate::physical_runtime::LiveIdempotencyBindingLimit,
    pub(super) generation: PhysicalNamespaceDurableCheckpointGeneration,
    pub(super) bindings:
        BTreeMap<PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdempotencyBindingState>,
}

pub(in crate::physical_runtime) enum PhysicalMutationIdempotencyRegistryAdmission {
    Fresh(UnallocatedPhysicalMutationAttemptBinding),
    DuplicateUnresolved(PhysicalMutationUnresolvedBindingObservation),
    Completed(Arc<CompletedPhysicalMutationFact>),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Indeterminate(IndeterminatePhysicalMutation),
}

pub(in crate::physical_runtime) enum PhysicalMutationIdempotencyRegistryAdmissionError<E> {
    Denied(PhysicalMutationIdempotencyRegistryDenial),
    Reservation(E),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalMutationIdempotencyRegistryDenial {
    AuthorityReleased,
    ForeignStore,
    ForeignPolicy,
    ForeignMutationStore,
    ForeignMutationRuntime,
    Expired,
    Conflict,
    PendingUnresolvedLimitReached,
    LiveBindingLimitReached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalMutationPreSealCancellationDenial {
    AuthorityReleased,
    BindingMismatch,
    GroupSealed,
    ReopenedUnresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalMutationIdempotencyGroupSealDenial {
    AuthorityReleased,
    BindingMismatch,
    AlreadyGroupSealed,
    ReopenedUnresolved,
    ProvenNoEffect,
}

impl PhysicalMutationIdempotencyGroupSealDenial {
    pub(in crate::physical_runtime) const fn admission_denial(
        self,
    ) -> crate::physical_runtime::PhysicalDurabilityGroupAdmissionDenial {
        use crate::physical_runtime::PhysicalDurabilityGroupAdmissionDenial as Denial;
        match self {
            Self::AuthorityReleased => Denial::IdempotencyAuthorityReleased,
            Self::BindingMismatch => Denial::IdempotencyBindingMismatch,
            Self::AlreadyGroupSealed => Denial::IdempotencyAlreadyGroupSealed,
            Self::ReopenedUnresolved => Denial::IdempotencyReopenedUnresolved,
            Self::ProvenNoEffect => Denial::IdempotencyProvenNoEffect,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalMutationWalBindingDenial {
    AuthorityReleased,
    BindingMismatch,
    AlreadyWalBound,
    ProvenNoEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalMutationTerminalizationDenial {
    AuthorityReleased,
    BindingMismatch,
    CompletionBeforeWalBinding,
    AlreadyTerminal,
}

impl PhysicalMutationIdempotencyRegistry {
    pub(super) fn generation_zero(
        store: StableStoreIdentity,
        runtime: RuntimeIdentity,
        policy: PhysicalDurabilityPolicyIdentity,
        idempotency: PhysicalIdempotencyPolicy,
    ) -> Self {
        Self {
            store,
            runtime,
            policy,
            retention: idempotency.retention(),
            pending_limit: idempotency.pending_unresolved_limit(),
            live_limit: idempotency.live_binding_limit(),
            generation: PhysicalNamespaceDurableCheckpointGeneration::INITIAL,
            bindings: BTreeMap::new(),
        }
    }

    pub(super) fn issue_key(
        &self,
        material: PhysicalMutationIdempotencyMaterial,
    ) -> Result<PhysicalMutationIdempotencyKey, PhysicalMutationLeaseIssuanceFailure> {
        let lease = PhysicalMutationIdempotencyLease::issue(
            self.store,
            self.policy,
            self.generation,
            self.retention,
        )?;
        Ok(PhysicalMutationIdempotencyKey::issue(lease, material))
    }

    pub(in crate::physical_runtime) fn cancel_before_group_seal(
        &mut self,
        expected: PhysicalMutationUnresolvedBindingObservation,
        cause: PhysicalMutationProvenNoEffectCause,
    ) -> Result<ProvenNoEffectPhysicalMutation, PhysicalMutationPreSealCancellationDenial> {
        let Some(state) = self.bindings.get_mut(&expected.key()) else {
            return Err(PhysicalMutationPreSealCancellationDenial::BindingMismatch);
        };
        match state {
            PhysicalMutationIdempotencyBindingState::Unsealed(basis)
                if basis.observation() == expected =>
            {
                let terminal = ProvenNoEffectPhysicalMutation::before_group_seal(
                    expected.key(),
                    expected.fingerprint(),
                    expected.mutation(),
                    cause,
                );
                *state = PhysicalMutationIdempotencyBindingState::Terminal {
                    basis: basis.clone(),
                    fate: PersistedPhysicalMutationFate::proven_no_effect(terminal),
                    last_compacted: None,
                };
                Ok(terminal)
            }
            PhysicalMutationIdempotencyBindingState::GroupSealed { basis, .. }
                if basis.observation() == expected =>
            {
                Err(PhysicalMutationPreSealCancellationDenial::GroupSealed)
            }
            PhysicalMutationIdempotencyBindingState::RebuiltUnresolved { basis, .. }
                if basis.observation() == expected =>
            {
                Err(PhysicalMutationPreSealCancellationDenial::ReopenedUnresolved)
            }
            PhysicalMutationIdempotencyBindingState::WalBound { basis, .. }
                if basis.observation() == expected =>
            {
                Err(PhysicalMutationPreSealCancellationDenial::GroupSealed)
            }
            PhysicalMutationIdempotencyBindingState::Terminal { basis, fate, .. }
                if basis.observation() == expected =>
            {
                fate.as_proven_no_effect()
                    .ok_or(PhysicalMutationPreSealCancellationDenial::GroupSealed)
            }
            _ => Err(PhysicalMutationPreSealCancellationDenial::BindingMismatch),
        }
    }

    pub(in crate::physical_runtime) fn seal_group(
        &mut self,
        expected: &[PhysicalMutationGroupSealingBinding],
    ) -> Result<(), PhysicalMutationIdempotencyGroupSealDenial> {
        for expected in expected {
            let observation = expected.observation();
            match self.bindings.get(&observation.key()) {
                Some(PhysicalMutationIdempotencyBindingState::Unsealed(basis))
                    if basis.observation() == observation => {}
                Some(PhysicalMutationIdempotencyBindingState::GroupSealed { basis, group })
                    if basis.observation() == observation && *group == expected.group() =>
                {
                    return Err(PhysicalMutationIdempotencyGroupSealDenial::AlreadyGroupSealed)
                }
                Some(PhysicalMutationIdempotencyBindingState::RebuiltUnresolved {
                    basis, ..
                }) if basis.observation() == observation => {
                    return Err(PhysicalMutationIdempotencyGroupSealDenial::ReopenedUnresolved)
                }
                Some(PhysicalMutationIdempotencyBindingState::Terminal { basis, .. })
                    if basis.observation() == observation =>
                {
                    return Err(PhysicalMutationIdempotencyGroupSealDenial::ProvenNoEffect)
                }
                _ => return Err(PhysicalMutationIdempotencyGroupSealDenial::BindingMismatch),
            }
        }
        for expected in expected {
            let observation = expected.observation();
            let state = self
                .bindings
                .get_mut(&observation.key())
                .expect("validated group bindings remain present under the registry lock");
            let PhysicalMutationIdempotencyBindingState::Unsealed(basis) = state else {
                unreachable!("validated group bindings remain unsealed until the update loop")
            };
            *state = PhysicalMutationIdempotencyBindingState::GroupSealed {
                basis: basis.clone(),
                group: expected.group(),
            };
        }
        Ok(())
    }

    pub(in crate::physical_runtime) fn record_wal_binding(
        &mut self,
        persisted: super::PersistedPhysicalMutationAttemptBinding,
    ) -> Result<(), PhysicalMutationWalBindingDenial> {
        let observation = persisted.observation();
        let Some(state) = self.bindings.get_mut(&observation.key()) else {
            return Err(PhysicalMutationWalBindingDenial::BindingMismatch);
        };
        match state {
            PhysicalMutationIdempotencyBindingState::GroupSealed { basis, group }
                if basis.observation() == observation && *group == persisted.group() =>
            {
                *state = PhysicalMutationIdempotencyBindingState::WalBound {
                    basis: basis.clone(),
                    persisted,
                };
                Ok(())
            }
            PhysicalMutationIdempotencyBindingState::RebuiltUnresolved { basis, prior } => {
                if basis.observation() != observation {
                    return Err(PhysicalMutationWalBindingDenial::BindingMismatch);
                }
                let prior_accepts_binding = match prior {
                    RebuiltPhysicalMutationBindingState::Unsealed => true,
                    RebuiltPhysicalMutationBindingState::GroupSealed(group) => {
                        *group == persisted.group()
                    }
                };
                if !prior_accepts_binding {
                    return Err(PhysicalMutationWalBindingDenial::BindingMismatch);
                }
                *state = PhysicalMutationIdempotencyBindingState::WalBound {
                    basis: basis.clone(),
                    persisted,
                };
                Ok(())
            }
            PhysicalMutationIdempotencyBindingState::WalBound {
                persisted: existing,
                ..
            } if existing == &persisted => Err(PhysicalMutationWalBindingDenial::AlreadyWalBound),
            PhysicalMutationIdempotencyBindingState::Terminal { .. } => {
                Err(PhysicalMutationWalBindingDenial::ProvenNoEffect)
            }
            _ => Err(PhysicalMutationWalBindingDenial::BindingMismatch),
        }
    }

    pub(in crate::physical_runtime) fn record_completed(
        &mut self,
        fact: Arc<CompletedPhysicalMutationFact>,
    ) -> Result<(), PhysicalMutationTerminalizationDenial> {
        let key = fact.idempotency_identity();
        let Some(state) = self.bindings.get_mut(&key) else {
            return Err(PhysicalMutationTerminalizationDenial::BindingMismatch);
        };
        match state {
            PhysicalMutationIdempotencyBindingState::WalBound { basis, persisted }
                if basis.fingerprint() == fact.request_fingerprint()
                    && basis.mutation() == fact.mutation_identity() =>
            {
                *state = PhysicalMutationIdempotencyBindingState::Terminal {
                    basis: basis.clone(),
                    fate: PersistedPhysicalMutationFate::completed(persisted.clone(), fact),
                    last_compacted: None,
                };
                Ok(())
            }
            PhysicalMutationIdempotencyBindingState::Terminal { .. } => {
                Err(PhysicalMutationTerminalizationDenial::AlreadyTerminal)
            }
            PhysicalMutationIdempotencyBindingState::Unsealed(_)
            | PhysicalMutationIdempotencyBindingState::GroupSealed { .. }
            | PhysicalMutationIdempotencyBindingState::RebuiltUnresolved { .. } => {
                Err(PhysicalMutationTerminalizationDenial::CompletionBeforeWalBinding)
            }
            PhysicalMutationIdempotencyBindingState::WalBound { .. } => {
                Err(PhysicalMutationTerminalizationDenial::BindingMismatch)
            }
        }
    }

    pub(in crate::physical_runtime) fn record_indeterminate(
        &mut self,
        terminal: IndeterminatePhysicalMutation,
    ) -> Result<(), PhysicalMutationTerminalizationDenial> {
        use super::fate::PersistedIndeterminatePhysicalMutationBasis;

        let key = terminal.idempotency_identity();
        let Some(state) = self.bindings.get_mut(&key) else {
            return Err(PhysicalMutationTerminalizationDenial::BindingMismatch);
        };
        let (basis, indeterminate_basis) = match state {
            PhysicalMutationIdempotencyBindingState::Unsealed(basis)
                if basis.matches_terminal(terminal) =>
            {
                (
                    basis.clone(),
                    PersistedIndeterminatePhysicalMutationBasis::Unsealed,
                )
            }
            PhysicalMutationIdempotencyBindingState::GroupSealed { basis, group }
                if basis.matches_terminal(terminal) =>
            {
                (
                    basis.clone(),
                    PersistedIndeterminatePhysicalMutationBasis::GroupSealed(*group),
                )
            }
            PhysicalMutationIdempotencyBindingState::WalBound { basis, persisted }
                if basis.matches_terminal(terminal) =>
            {
                (
                    basis.clone(),
                    PersistedIndeterminatePhysicalMutationBasis::WalBound(persisted.clone()),
                )
            }
            PhysicalMutationIdempotencyBindingState::Terminal { .. } => {
                return Err(PhysicalMutationTerminalizationDenial::AlreadyTerminal)
            }
            _ => return Err(PhysicalMutationTerminalizationDenial::BindingMismatch),
        };
        *state = PhysicalMutationIdempotencyBindingState::Terminal {
            basis,
            fate: PersistedPhysicalMutationFate::indeterminate(indeterminate_basis, terminal),
            last_compacted: None,
        };
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn set_namespace_durable_generation_for_test(&mut self, generation: u64) {
        self.generation =
            PhysicalNamespaceDurableCheckpointGeneration::from_namespace_durable_checkpoint(
                generation,
            );
    }
}

#[cfg(test)]
#[path = "registry/tests.rs"]
mod tests;
