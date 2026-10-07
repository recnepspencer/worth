//! Registry-issued evidence for one original C.11 drop's negative fate.

use super::{
    binding_compaction::drop_material,
    fate::DuplicatePhysicalMutationTerminal,
    key::PhysicalMutationIdempotencyKeyIdentity,
    registry::{PhysicalMutationIdempotencyBindingState, PhysicalMutationIdempotencyRegistry},
};
use worth_store_physical_format::{
    OriginalDropReservedV1, PersistedRecordIdentity, RootPublicationCell,
};

use super::{
    PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyLease,
    PhysicalMutationIdempotencyMaterial,
};

/// Scoped eligibility evidence from a fully rebuilt registry. Store must
/// join this to the exact selected reservation under session/root custody;
/// it never authorizes replay of the precommitted original request.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct PhysicalRecoveredOriginalDropNoDurableEffect {
    reservation_record: PersistedRecordIdentity,
    reservation_sha256: [u8; 32],
    reservation: OriginalDropReservedV1,
    selected_root: RootPublicationCell,
    registry_runtime: crate::physical_runtime::RuntimeIdentity,
}

impl PhysicalRecoveredOriginalDropNoDurableEffect {
    pub(in crate::physical_runtime) const fn reservation_record(self) -> PersistedRecordIdentity {
        self.reservation_record
    }

    pub(in crate::physical_runtime) const fn reservation_sha256(self) -> [u8; 32] {
        self.reservation_sha256
    }

    pub(in crate::physical_runtime) const fn reservation(self) -> OriginalDropReservedV1 {
        self.reservation
    }

    pub(in crate::physical_runtime) const fn selected_root(self) -> RootPublicationCell {
        self.selected_root
    }

    pub(in crate::physical_runtime) const fn registry_runtime(
        self,
    ) -> crate::physical_runtime::RuntimeIdentity {
        self.registry_runtime
    }
}

#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct PhysicalOriginalDropNoEffect {
    store: [u8; 16],
    attempt: [u8; 16],
    idempotency: [u8; 32],
    fingerprint: [u8; 32],
}

/// Registry-sealed completion of the original descriptor named by a selected
/// C.11 reservation. The selected descriptor frame must still be joined by
/// the caller; this token is not record content authority on its own.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct PhysicalOriginalDropCompleted {
    idempotency: [u8; 32],
    fingerprint: [u8; 32],
    descriptor: PersistedRecordIdentity,
    current_root_generation: u64,
}

impl PhysicalOriginalDropCompleted {
    pub(in crate::physical_runtime) const fn idempotency_identity(self) -> [u8; 32] {
        self.idempotency
    }

    pub(in crate::physical_runtime) const fn request_fingerprint(self) -> [u8; 32] {
        self.fingerprint
    }

    pub(in crate::physical_runtime) const fn descriptor_record(self) -> PersistedRecordIdentity {
        self.descriptor
    }

    pub(in crate::physical_runtime) const fn current_root_generation(self) -> u64 {
        self.current_root_generation
    }
}

impl PhysicalOriginalDropNoEffect {
    pub(in crate::physical_runtime) const fn store(self) -> [u8; 16] {
        self.store
    }

    pub(in crate::physical_runtime) const fn attempt(self) -> [u8; 16] {
        self.attempt
    }

    pub(in crate::physical_runtime) const fn idempotency_identity(self) -> [u8; 32] {
        self.idempotency
    }

    pub(in crate::physical_runtime) const fn request_fingerprint(self) -> [u8; 32] {
        self.fingerprint
    }
}

impl PhysicalMutationIdempotencyRegistry {
    pub(super) fn discover_recovered_original_drop_no_durable_effect(
        &self,
        reservation_record: PersistedRecordIdentity,
        reservation_sha256: [u8; 32],
        reservation: OriginalDropReservedV1,
        selected_root: RootPublicationCell,
    ) -> Option<PhysicalRecoveredOriginalDropNoDurableEffect> {
        if !self.recovered_complete
            || reservation_sha256 == [0; 32]
            || reservation_record == reservation.manifest_record()
            || reservation.store() != self.store_identity().bytes()
            || reservation.reserved_selected_generation() > selected_root.generation().get()
            || reservation.request().lease_issuance_generation() > self.generation.get()
            || self.original_drop_binding_absent(reservation.store(), reservation.reclaim_attempt())
                != Some(true)
        {
            return None;
        }
        let request = reservation.request();
        let lease = PhysicalMutationIdempotencyLease::from_reopened(
            self.store_identity(),
            self.policy_identity(),
            request.lease_issuance_generation(),
            request.lease_expiry_generation(),
            self.retention(),
        )?;
        let key = PhysicalMutationIdempotencyKey::issue(
            lease,
            PhysicalMutationIdempotencyMaterial::new(drop_material(
                reservation.store(),
                reservation.reclaim_attempt(),
            )),
        );
        if key.identity().bytes() != request.idempotency() {
            return None;
        }
        Some(PhysicalRecoveredOriginalDropNoDurableEffect {
            reservation_record,
            reservation_sha256,
            reservation,
            selected_root,
            registry_runtime: self.runtime_identity(),
        })
    }

    /// Veto-only observation for selected NeverReserved custody. An absent
    /// binding is not itself authority that the original descriptor had no
    /// durable effect; the V2 selected slot and full selected scan supply that.
    pub(super) fn original_drop_binding_absent(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
    ) -> Option<bool> {
        if self.store_identity().bytes() != store || attempt == [0; 16] {
            return None;
        }
        let material = drop_material(store, attempt);
        Some(!self.bindings.values().any(|state| {
            let basis = match state {
                PhysicalMutationIdempotencyBindingState::Unsealed(basis)
                | PhysicalMutationIdempotencyBindingState::GroupSealed { basis, .. }
                | PhysicalMutationIdempotencyBindingState::RebuiltUnresolved { basis, .. }
                | PhysicalMutationIdempotencyBindingState::WalBound { basis, .. }
                | PhysicalMutationIdempotencyBindingState::Terminal { basis, .. } => basis,
            };
            basis.key().caller_material().bytes() == material
        }))
    }

    pub(super) fn discover_original_drop_completed(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
    ) -> Option<PhysicalOriginalDropCompleted> {
        if self.store_identity().bytes() != store || attempt == [0; 16] {
            return None;
        }
        let material = drop_material(store, attempt);
        let mut completed = None;
        for state in self.bindings.values() {
            let basis = match state {
                PhysicalMutationIdempotencyBindingState::Unsealed(basis)
                | PhysicalMutationIdempotencyBindingState::GroupSealed { basis, .. }
                | PhysicalMutationIdempotencyBindingState::RebuiltUnresolved { basis, .. }
                | PhysicalMutationIdempotencyBindingState::WalBound { basis, .. }
                | PhysicalMutationIdempotencyBindingState::Terminal { basis, .. } => basis,
            };
            if basis.key().caller_material().bytes() != material {
                continue;
            }
            if completed.is_some()
                || basis.key().identity().bytes() != idempotency
                || basis.key().lease().store_identity().bytes() != store
                || basis.fingerprint().bytes() != fingerprint
            {
                return None;
            }
            let PhysicalMutationIdempotencyBindingState::Terminal { fate, .. } = state else {
                return None;
            };
            let DuplicatePhysicalMutationTerminal::Completed(fact) =
                fate.duplicate_observation(basis.fingerprint())?
            else {
                return None;
            };
            let [descriptor] = fact.persisted_records() else {
                return None;
            };
            if fact.idempotency_identity().bytes() != idempotency
                || fact.request_fingerprint().bytes() != fingerprint
                || fact.mutation_identity() != basis.mutation()
            {
                return None;
            }
            completed = Some(PhysicalOriginalDropCompleted {
                idempotency,
                fingerprint,
                descriptor: *descriptor,
                current_root_generation: fact.breadth().current_root_generation(),
            });
        }
        completed
    }

    /// Discover the canonical stage-two binding under registry authority.
    /// A second lease/binding for the same caller material is ambiguous even
    /// if one of the candidates has a negative fate.
    pub(super) fn discover_original_drop_no_effect(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
    ) -> Option<PhysicalOriginalDropNoEffect> {
        if self.store_identity().bytes() != store || attempt == [0; 16] {
            return None;
        }
        let material = drop_material(store, attempt);
        let mut candidate = None;
        for state in self.bindings.values() {
            let basis = match state {
                PhysicalMutationIdempotencyBindingState::Unsealed(basis)
                | PhysicalMutationIdempotencyBindingState::GroupSealed { basis, .. }
                | PhysicalMutationIdempotencyBindingState::RebuiltUnresolved { basis, .. }
                | PhysicalMutationIdempotencyBindingState::WalBound { basis, .. }
                | PhysicalMutationIdempotencyBindingState::Terminal { basis, .. } => basis,
            };
            if basis.key().caller_material().bytes() != material {
                continue;
            }
            if candidate.is_some() {
                return None;
            }
            let PhysicalMutationIdempotencyBindingState::Terminal { fate, .. } = state else {
                return None;
            };
            if fate.as_proven_no_effect().is_none() {
                return None;
            }
            candidate = Some((basis.key().identity().bytes(), basis.fingerprint().bytes()));
        }
        let (idempotency, fingerprint) = candidate?;
        self.reconcile_original_drop_no_effect(store, attempt, idempotency, fingerprint)
    }

    pub(super) fn reconcile_original_drop_no_effect(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
    ) -> Option<PhysicalOriginalDropNoEffect> {
        if self.store_identity().bytes() != store || attempt == [0; 16] {
            return None;
        }
        let PhysicalMutationIdempotencyBindingState::Terminal { basis, fate, .. } =
            self.bindings
                .get(&PhysicalMutationIdempotencyKeyIdentity::from_observed_bytes(idempotency))?
        else {
            return None;
        };
        let terminal = fate.as_proven_no_effect()?;
        if basis.key().identity().bytes() != idempotency
            || basis.key().lease().store_identity().bytes() != store
            || basis.key().caller_material().bytes() != drop_material(store, attempt)
            || basis.fingerprint().bytes() != fingerprint
            || terminal.idempotency_identity().bytes() != idempotency
            || terminal.request_fingerprint().bytes() != fingerprint
            || terminal.mutation_identity() != basis.mutation()
        {
            return None;
        }
        Some(PhysicalOriginalDropNoEffect {
            store,
            attempt,
            idempotency,
            fingerprint,
        })
    }
}

#[cfg(test)]
mod absence_tests {
    use super::*;
    use crate::physical_runtime::durability::mutation::idempotency::test_support;
    use crate::physical_runtime::PhysicalMutationIdempotencyMaterial;

    #[test]
    fn every_original_drop_binding_state_vetoes_never_reserved() {
        let mut fixture = test_support::fixture(2);
        let store = fixture.store.bytes();
        let attempt = [7; 16];
        assert_eq!(
            fixture
                .registry
                .original_drop_binding_absent(store, attempt),
            Some(true)
        );
        assert_eq!(
            fixture
                .registry
                .original_drop_binding_absent([9; 16], attempt),
            None
        );
        let key = fixture
            .registry
            .issue_key(PhysicalMutationIdempotencyMaterial::new(drop_material(
                store, attempt,
            )))
            .unwrap();
        let fingerprint = test_support::fingerprint(&fixture, 1);
        let mutation = test_support::mutation(&fixture, 1);
        assert!(matches!(
            fixture
                .registry
                .admit_unallocated(key, fingerprint, mutation),
            Ok(super::super::registry::PhysicalMutationIdempotencyRegistryAdmission::Fresh(_))
        ));
        assert_eq!(
            fixture
                .registry
                .original_drop_binding_absent(store, attempt),
            Some(false)
        );
        fixture.media.close();
    }
}
