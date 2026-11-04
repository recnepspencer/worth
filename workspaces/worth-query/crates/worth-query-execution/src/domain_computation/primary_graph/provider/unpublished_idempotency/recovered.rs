//! Query key admission retains original cold source meaning, never a live binding.
use super::super::idempotency::{
    snapshot_resolution::RecordedOriginalBinding,
    WorthQueryProviderIdempotencyResolutionDenial as Denial,
};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryRecordedIntentMatch;

pub(super) struct RecoveredIdempotencyEntry {
    binding: RecordedOriginalBinding,
    handle: ProductUnpublishedRecoveryHandle,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl super::super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn recovered_unpublished_application(
        &self,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        handle: ProductUnpublishedRecoveryHandle,
    ) -> Result<
        crate::domain_computation::WorthQueryProductUnpublishedApplication,
        worth_runtime_world::facade::RuntimeWorldRecoveryDenial,
    > {
        let recovery = product.recovery();
        let effects = recovery.inspect_recovered_effects(&handle)?;
        Ok(
            crate::domain_computation::WorthQueryProductUnpublishedApplication::new_recovered(
                effects,
                recovery,
                self.unpublished_idempotency_disposition(),
            ),
        )
    }
    pub(in crate::domain_computation::primary_graph) fn admit_recovered_idempotency(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: ProductUnpublishedRecoveryHandle,
        effects: &worth_runtime_world::facade::WorldRecoveredUnpublishedEffects,
    ) -> Result<(), Denial> {
        let Some(native) = effects.native_effect() else {
            return Ok(());
        };
        let layout = self.graph.layout.provider_idempotency();
        let mut original = None;
        for created in native.semantic_result().created_entities() {
            if created.key.kind != layout.entity_kind {
                continue;
            }
            if original.is_some() {
                return Err(Denial::Unavailable);
            }
            let row = native
                .created_entity(execution, created.assigned)
                .map_err(|_| Denial::Unavailable)?
                .ok_or(Denial::Unavailable)?;
            let funding = execution
                .reserve_retained_memory(
                    row.owned_allocation_capacity_bytes()
                        .checked_mul(2)
                        .ok_or(Denial::Unavailable)?
                        .checked_add(std::mem::size_of::<RecoveredIdempotencyEntry>() as u64)
                        .ok_or(Denial::Unavailable)?,
                )
                .map_err(|_| Denial::RetentionCapacityExhausted)?;
            let binding = super::super::idempotency::snapshot_resolution::original_binding::read(
                &row, layout,
            )?;
            original = Some(RecoveredIdempotencyEntry {
                binding,
                handle: handle.clone(),
                _funding: funding,
            });
        }
        let original = original.ok_or(Denial::Unavailable)?;
        let affinity =
            WorthQueryProductIdempotencyAffinity::from_observation(effects.expected_head());
        if original.binding.affinity
            != super::super::idempotency::recorded_origin::affinity_text(&affinity)
        {
            return Err(Denial::ForeignAffinity);
        }
        let key = WorthQueryUnpublishedIdempotencyKey::for_original(
            affinity,
            original.binding.key,
            &original.binding.intent,
        )?;
        let mut store = self
            .unpublished_idempotency
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if store.by_key.contains_key(&key)
            || store.recovered.contains_key(&key)
            || store
                .maximum_entries
                .is_some_and(|maximum| store.by_key.len() + store.recovered.len() >= maximum)
        {
            return Err(Denial::Unavailable);
        }
        store.recovered.insert(key, original);
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn resolve_recovered_application_idempotency(
        &self,
        product: &WorthQueryProductIdempotencyAffinity,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<Option<WorthQueryProviderIdempotencyResolution>, Denial> {
        let store = self
            .unpublished_idempotency
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let key = WorthQueryUnpublishedIdempotencyKey::for_binding(product, &binding);
        let Some(entry) = store.recovered.get(&key) else {
            return Ok(None);
        };
        Ok(Some(
            match binding.match_recorded_intent(&entry.binding.intent)? {
                WorthQueryRecordedIntentMatch::Same => {
                    WorthQueryProviderIdempotencyResolution::RecoveredUnpublished {
                        handle: entry.handle.clone(),
                    }
                }
                WorthQueryRecordedIntentMatch::Drift => {
                    WorthQueryProviderIdempotencyResolution::Drift
                }
                WorthQueryRecordedIntentMatch::Unverifiable => {
                    return Err(Denial::RecordedIntentUnverifiable)
                }
            },
        ))
    }
}
impl WorthQueryUnpublishedIdempotencyDisposition {
    pub(in crate::domain_computation) fn finish_recovered(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) {
        self.store
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .recovered
            .retain(|_, entry| &entry.handle != handle);
    }
}
