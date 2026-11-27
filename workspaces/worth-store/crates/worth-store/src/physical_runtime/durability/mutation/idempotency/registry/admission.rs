use super::super::attempt_binding::{PhysicalMutationAttemptBinding, WalUnallocated};
use super::{
    PhysicalMutationBindingBasis, PhysicalMutationIdempotencyBindingState,
    PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyRegistry,
    PhysicalMutationIdempotencyRegistryAdmission,
    PhysicalMutationIdempotencyRegistryAdmissionError, PhysicalMutationIdempotencyRegistryDenial,
    PhysicalMutationIdentity, PhysicalMutationRequestFingerprint,
};

impl PhysicalMutationIdempotencyRegistry {
    #[cfg(test)]
    pub(in crate::physical_runtime::durability::mutation::idempotency) fn set_namespace_durable_generation_for_test(
        &mut self,
        generation: u64,
    ) {
        self.generation =
            super::PhysicalNamespaceDurableCheckpointGeneration::from_namespace_durable_checkpoint(
                generation,
            );
        self.admitted_since_checkpoint = 0;
    }

    #[cfg(test)]
    pub(in crate::physical_runtime::durability::mutation::idempotency) fn admit_unallocated(
        &mut self,
        key: PhysicalMutationIdempotencyKey,
        fingerprint: PhysicalMutationRequestFingerprint,
        mutation: PhysicalMutationIdentity,
    ) -> Result<
        PhysicalMutationIdempotencyRegistryAdmission,
        PhysicalMutationIdempotencyRegistryDenial,
    > {
        match self.admit_unallocated_with(key, fingerprint, || {
            Ok::<_, std::convert::Infallible>(mutation)
        }) {
            Ok(admission) => Ok(admission),
            Err(PhysicalMutationIdempotencyRegistryAdmissionError::Denied(denial)) => Err(denial),
            Err(PhysicalMutationIdempotencyRegistryAdmissionError::Reservation(never)) => {
                match never {}
            }
        }
    }

    pub(in crate::physical_runtime) fn admit_unallocated_with<E>(
        &mut self,
        key: PhysicalMutationIdempotencyKey,
        fingerprint: PhysicalMutationRequestFingerprint,
        reserve: impl FnOnce() -> Result<PhysicalMutationIdentity, E>,
    ) -> Result<
        PhysicalMutationIdempotencyRegistryAdmission,
        PhysicalMutationIdempotencyRegistryAdmissionError<E>,
    > {
        self.validate_key_authority(&key)?;
        if let Some(existing) = self.bindings.get(&key.identity()) {
            return classify_existing_binding(existing, fingerprint);
        }
        self.validate_fresh_admission(&key)?;
        let mutation =
            reserve().map_err(PhysicalMutationIdempotencyRegistryAdmissionError::Reservation)?;
        self.validate_reserved_mutation(mutation)?;
        let basis = PhysicalMutationBindingBasis::new(key.clone(), fingerprint, mutation);
        self.bindings.insert(
            key.identity(),
            PhysicalMutationIdempotencyBindingState::Unsealed(basis),
        );
        self.admitted_since_checkpoint = self.admitted_since_checkpoint.saturating_add(1);
        Ok(PhysicalMutationIdempotencyRegistryAdmission::Fresh(
            PhysicalMutationAttemptBinding::<WalUnallocated>::new(key, fingerprint, mutation),
        ))
    }

    fn validate_key_authority<E>(
        &self,
        key: &PhysicalMutationIdempotencyKey,
    ) -> Result<(), PhysicalMutationIdempotencyRegistryAdmissionError<E>> {
        if key.lease().store_identity() != self.store {
            return denied(PhysicalMutationIdempotencyRegistryDenial::ForeignStore);
        }
        if key.lease().policy_identity() != self.policy {
            return denied(PhysicalMutationIdempotencyRegistryDenial::ForeignPolicy);
        }
        Ok(())
    }

    fn validate_fresh_admission<E>(
        &self,
        key: &PhysicalMutationIdempotencyKey,
    ) -> Result<(), PhysicalMutationIdempotencyRegistryAdmissionError<E>> {
        if key.lease().is_expired_at(self.generation) {
            return denied(PhysicalMutationIdempotencyRegistryDenial::Expired);
        }
        if self.bindings.len() >= self.live_limit.get().get() as usize {
            return denied(PhysicalMutationIdempotencyRegistryDenial::LiveBindingLimitReached);
        }
        if self.pending_binding_count() >= self.pending_limit.get().get() as usize {
            return denied(
                PhysicalMutationIdempotencyRegistryDenial::PendingUnresolvedLimitReached,
            );
        }
        Ok(())
    }

    fn validate_reserved_mutation<E>(
        &self,
        mutation: PhysicalMutationIdentity,
    ) -> Result<(), PhysicalMutationIdempotencyRegistryAdmissionError<E>> {
        if mutation.store_identity() != self.store {
            return denied(PhysicalMutationIdempotencyRegistryDenial::ForeignMutationStore);
        }
        if mutation.runtime_identity() != self.runtime {
            return denied(PhysicalMutationIdempotencyRegistryDenial::ForeignMutationRuntime);
        }
        Ok(())
    }

    fn pending_binding_count(&self) -> usize {
        self.bindings
            .values()
            .filter(|state| state.holds_pending_capacity())
            .count()
    }
}

fn classify_existing_binding<E>(
    existing: &PhysicalMutationIdempotencyBindingState,
    fingerprint: PhysicalMutationRequestFingerprint,
) -> Result<
    PhysicalMutationIdempotencyRegistryAdmission,
    PhysicalMutationIdempotencyRegistryAdmissionError<E>,
> {
    match existing {
        PhysicalMutationIdempotencyBindingState::Unsealed(existing)
        | PhysicalMutationIdempotencyBindingState::GroupSealed {
            basis: existing, ..
        }
        | PhysicalMutationIdempotencyBindingState::RebuiltUnresolved {
            basis: existing, ..
        }
        | PhysicalMutationIdempotencyBindingState::WalBound {
            basis: existing, ..
        } if existing.fingerprint() == fingerprint => Ok(
            PhysicalMutationIdempotencyRegistryAdmission::DuplicateUnresolved(
                existing.observation(),
            ),
        ),
        PhysicalMutationIdempotencyBindingState::Terminal { fate, .. } => {
            let terminal = fate.duplicate_observation(fingerprint).ok_or_else(|| {
                PhysicalMutationIdempotencyRegistryAdmissionError::Denied(
                    PhysicalMutationIdempotencyRegistryDenial::Conflict,
                )
            })?;
            Ok(match terminal {
                super::super::fate::DuplicatePhysicalMutationTerminal::Completed(fact) => {
                    PhysicalMutationIdempotencyRegistryAdmission::Completed(fact)
                }
                super::super::fate::DuplicatePhysicalMutationTerminal::ProvenNoEffect(fate) => {
                    PhysicalMutationIdempotencyRegistryAdmission::ProvenNoEffect(fate)
                }
                super::super::fate::DuplicatePhysicalMutationTerminal::Indeterminate(fate) => {
                    PhysicalMutationIdempotencyRegistryAdmission::Indeterminate(fate)
                }
            })
        }
        _ => denied(PhysicalMutationIdempotencyRegistryDenial::Conflict),
    }
}

fn denied<T, E>(
    denial: PhysicalMutationIdempotencyRegistryDenial,
) -> Result<T, PhysicalMutationIdempotencyRegistryAdmissionError<E>> {
    Err(PhysicalMutationIdempotencyRegistryAdmissionError::Denied(
        denial,
    ))
}
