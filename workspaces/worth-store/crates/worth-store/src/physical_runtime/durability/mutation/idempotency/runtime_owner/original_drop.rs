//! Runtime registry facade for original-drop idempotency observations.

use super::*;

impl PhysicalMutationIdempotencyRuntimeAuthority {
    pub(in crate::physical_runtime) fn discover_recovered_original_drop_no_durable_effect(
        &self,
        reservation_record: worth_store_physical_format::PersistedRecordIdentity,
        reservation_sha256: [u8; 32],
        reservation: worth_store_physical_format::OriginalDropReservedV1,
        selected_root: worth_store_physical_format::RootPublicationCell,
    ) -> Option<super::super::PhysicalRecoveredOriginalDropNoDurableEffect> {
        let owner = self.owner.upgrade()?;
        let result = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .discover_recovered_original_drop_no_durable_effect(
                reservation_record,
                reservation_sha256,
                reservation,
                selected_root,
            );
        result
    }

    pub(in crate::physical_runtime) fn original_drop_binding_absent(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
    ) -> Option<bool> {
        let owner = self.owner.upgrade()?;
        let absent = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .original_drop_binding_absent(store, attempt);
        absent
    }

    pub(in crate::physical_runtime) fn discover_original_drop_completed(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
    ) -> Option<super::super::PhysicalOriginalDropCompleted> {
        let owner = self.owner.upgrade()?;
        let completed = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .discover_original_drop_completed(store, attempt, idempotency, fingerprint);
        completed
    }

    pub(in crate::physical_runtime) fn discover_original_drop_no_effect(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
    ) -> Option<super::super::PhysicalOriginalDropNoEffect> {
        let owner = self.owner.upgrade()?;
        let result = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .discover_original_drop_no_effect(store, attempt);
        result
    }

    pub(in crate::physical_runtime) fn reconcile_original_drop_no_effect(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
    ) -> Option<super::super::PhysicalOriginalDropNoEffect> {
        let owner = self.owner.upgrade()?;
        let result = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reconcile_original_drop_no_effect(store, attempt, idempotency, fingerprint);
        result
    }
}
