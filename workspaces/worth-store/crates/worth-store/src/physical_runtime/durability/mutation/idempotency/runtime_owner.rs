use std::sync::{Arc, Mutex, MutexGuard, Weak};

use super::{
    binding_compaction::{
        PendingPhysicalMutationBindingCompaction, PhysicalMutationBindingCompaction,
        PhysicalMutationBindingCompactionDenial,
    },
    lease::PhysicalMutationLeaseIssuanceFailure,
    registry::{
        PhysicalMutationGroupSealingBinding, PhysicalMutationIdempotencyGroupSealDenial,
        PhysicalMutationIdempotencyRegistry, PhysicalMutationIdempotencyRegistryAdmission,
        PhysicalMutationIdempotencyRegistryAdmissionError,
        PhysicalMutationIdempotencyRegistryDenial, PhysicalMutationPreSealCancellationDenial,
        PhysicalMutationTerminalizationDenial, PhysicalMutationUnresolvedBindingObservation,
        PhysicalMutationWalBindingDenial,
    },
    PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyMaterial,
};

pub(in crate::physical_runtime) struct PhysicalMutationIdempotencyRuntimeOwner {
    registry: Mutex<PhysicalMutationIdempotencyRegistry>,
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalMutationIdempotencyRuntimeAuthority {
    owner: Weak<PhysicalMutationIdempotencyRuntimeOwner>,
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalMutationBindingCompactionRuntimeAuthority {
    owner: Arc<PhysicalMutationIdempotencyRuntimeOwner>,
}

pub(in crate::physical_runtime) struct PhysicalMutationBindingCompactionCutover<'owner> {
    registry: MutexGuard<'owner, PhysicalMutationIdempotencyRegistry>,
    pending: Option<PendingPhysicalMutationBindingCompaction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationIdempotencyIssuanceDenial {
    DurabilityAuthorityReleased,
    LeaseGenerationExhausted,
}

impl PhysicalMutationIdempotencyRuntimeOwner {
    pub(super) fn from_rebuilt_registry(
        registry: PhysicalMutationIdempotencyRegistry,
    ) -> Arc<Self> {
        Arc::new(Self {
            registry: Mutex::new(registry),
        })
    }

    pub(in crate::physical_runtime) fn authority(
        owner: &Arc<Self>,
    ) -> PhysicalMutationIdempotencyRuntimeAuthority {
        PhysicalMutationIdempotencyRuntimeAuthority {
            owner: Arc::downgrade(owner),
        }
    }

    pub(in crate::physical_runtime) fn binding_compaction_authority(
        owner: &Arc<Self>,
    ) -> PhysicalMutationBindingCompactionRuntimeAuthority {
        PhysicalMutationBindingCompactionRuntimeAuthority {
            owner: Arc::clone(owner),
        }
    }

    pub(in crate::physical_runtime) fn into_recovery_operation_fates(
        owner: Arc<Self>,
        completed_unobserved: Box<[crate::physical_runtime::CompletedUnobservedPhysicalMutation]>,
    ) -> Result<
        crate::physical_runtime::PhysicalRecoveryOperationFates,
        crate::physical_runtime::durability::PhysicalIdempotencyCloseoutDenial,
    > {
        let owner = Arc::try_unwrap(owner).map_err(|_| {
            crate::physical_runtime::durability::PhysicalIdempotencyCloseoutDenial::LiveCompactionAuthority
        })?;
        let registry = owner
            .registry
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(
            crate::physical_runtime::PhysicalRecoveryOperationFates::new(
                registry,
                completed_unobserved,
            ),
        )
    }
}

impl PhysicalMutationBindingCompactionRuntimeAuthority {
    pub(in crate::physical_runtime) fn begin_binding_compaction(
        &self,
        checkpoint: worth_store_physical_format::PhysicalCheckpointIdentity,
        wal_cutoff_lsn_exclusive: u64,
    ) -> Result<PhysicalMutationBindingCompactionCutover<'_>, PhysicalMutationBindingCompactionDenial>
    {
        let registry = self
            .owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let pending = registry.prepare_binding_compaction(checkpoint, wal_cutoff_lsn_exclusive)?;
        Ok(PhysicalMutationBindingCompactionCutover {
            registry,
            pending: Some(pending),
        })
    }

    /// Reacquires the registry after a media barrier. Foreground mutations may
    /// have changed bindings; only a committed generation change invalidates
    /// the prospective compaction.
    pub(in crate::physical_runtime) fn resume_binding_compaction(
        &self,
        pending: PendingPhysicalMutationBindingCompaction,
    ) -> Result<PhysicalMutationBindingCompactionCutover<'_>, PhysicalMutationBindingCompactionDenial>
    {
        let registry = self
            .owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if registry.generation != pending.prior_generation() {
            return Err(PhysicalMutationBindingCompactionDenial::RegistryChanged);
        }
        Ok(PhysicalMutationBindingCompactionCutover {
            registry,
            pending: Some(pending),
        })
    }
}

impl PhysicalMutationIdempotencyRuntimeAuthority {
    pub(in crate::physical_runtime) fn record_completed(
        &self,
        fact: Arc<crate::physical_runtime::CompletedPhysicalMutationFact>,
    ) -> Result<(), PhysicalMutationTerminalizationDenial> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(PhysicalMutationTerminalizationDenial::AuthorityReleased)?;
        let outcome = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record_completed(fact);
        outcome
    }

    pub(in crate::physical_runtime) fn record_indeterminate(
        &self,
        fate: crate::physical_runtime::IndeterminatePhysicalMutation,
    ) -> Result<(), PhysicalMutationTerminalizationDenial> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(PhysicalMutationTerminalizationDenial::AuthorityReleased)?;
        let outcome = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record_indeterminate(fate);
        outcome
    }

    pub(in crate::physical_runtime) fn record_wal_binding(
        &self,
        persisted: super::PersistedPhysicalMutationAttemptBinding,
    ) -> Result<(), PhysicalMutationWalBindingDenial> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(PhysicalMutationWalBindingDenial::AuthorityReleased)?;
        let outcome = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record_wal_binding(persisted);
        outcome
    }

    pub(in crate::physical_runtime) fn issue_key(
        &self,
        material: PhysicalMutationIdempotencyMaterial,
    ) -> Result<PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyIssuanceDenial> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(PhysicalMutationIdempotencyIssuanceDenial::DurabilityAuthorityReleased)?;
        let registry = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        registry
            .issue_key(material)
            .map_err(|failure| match failure {
                PhysicalMutationLeaseIssuanceFailure::GenerationExhausted => {
                    PhysicalMutationIdempotencyIssuanceDenial::LeaseGenerationExhausted
                }
            })
    }

    pub(in crate::physical_runtime) fn admit_unallocated_with<E>(
        &self,
        key: PhysicalMutationIdempotencyKey,
        fingerprint: crate::physical_runtime::PhysicalMutationRequestFingerprint,
        reserve: impl FnOnce() -> Result<crate::physical_runtime::PhysicalMutationIdentity, E>,
    ) -> Result<
        PhysicalMutationIdempotencyRegistryAdmission,
        PhysicalMutationIdempotencyRegistryAdmissionError<E>,
    > {
        let owner = self.owner.upgrade().ok_or(
            PhysicalMutationIdempotencyRegistryAdmissionError::Denied(
                PhysicalMutationIdempotencyRegistryDenial::AuthorityReleased,
            ),
        )?;
        let admission = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .admit_unallocated_with(key, fingerprint, reserve);
        admission
    }

    pub(in crate::physical_runtime) fn cancel_before_group_seal(
        &self,
        expected: PhysicalMutationUnresolvedBindingObservation,
        cause: crate::physical_runtime::PhysicalMutationProvenNoEffectCause,
    ) -> Result<
        crate::physical_runtime::ProvenNoEffectPhysicalMutation,
        PhysicalMutationPreSealCancellationDenial,
    > {
        let owner = self
            .owner
            .upgrade()
            .ok_or(PhysicalMutationPreSealCancellationDenial::AuthorityReleased)?;
        let outcome = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .cancel_before_group_seal(expected, cause);
        outcome
    }

    pub(in crate::physical_runtime) fn seal_group(
        &self,
        expected: &[PhysicalMutationGroupSealingBinding],
    ) -> Result<(), PhysicalMutationIdempotencyGroupSealDenial> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(PhysicalMutationIdempotencyGroupSealDenial::AuthorityReleased)?;
        let outcome = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .seal_group(expected);
        outcome
    }
}

impl PhysicalMutationBindingCompactionCutover<'_> {
    pub(in crate::physical_runtime) fn generation(
        &self,
    ) -> super::PhysicalNamespaceDurableCheckpointGeneration {
        self.pending_ref().generation()
    }

    pub(in crate::physical_runtime) fn wal_cutoff_lsn_exclusive(&self) -> u64 {
        self.pending_ref().wal_cutoff_lsn_exclusive()
    }

    pub(in crate::physical_runtime) fn for_each_record<E>(
        &self,
        consume: impl FnMut(&[u8]) -> Result<(), E>,
    ) -> Result<(), E> {
        self.pending_ref().for_each_record(&self.registry, consume)
    }

    /// Drops the registry guard so a non-preemptible media barrier cannot stall
    /// foreground mutations that record a WAL binding under this same mutex.
    pub(in crate::physical_runtime) fn release_registry(
        mut self,
    ) -> PendingPhysicalMutationBindingCompaction {
        self.pending
            .take()
            .expect("an uncommitted cutover retains its prospective compaction")
    }

    pub(in crate::physical_runtime) fn commit_namespace_durable(
        mut self,
        namespace_sync: &crate::physical_runtime::work::CompletedPhysicalCheckpointAction,
    ) -> Result<PhysicalMutationBindingCompaction, PhysicalMutationBindingCompactionDenial> {
        self.pending
            .take()
            .expect("a move-owned cutover commits at most once")
            .commit(&mut self.registry, namespace_sync)
    }

    fn pending_ref(&self) -> &PendingPhysicalMutationBindingCompaction {
        self.pending
            .as_ref()
            .expect("an uncommitted cutover retains its prospective compaction")
    }
}
