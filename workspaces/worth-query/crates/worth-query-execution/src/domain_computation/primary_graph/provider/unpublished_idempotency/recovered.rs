//! Original source admission is retryable and distinct from live retained results.
use super::super::idempotency::{
    released_original::{self, OriginalBinding, WorthQueryReleasedApplicationEffect},
    WorthQueryProviderIdempotencyResolutionDenial as Denial,
};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryRecordedIntentMatch;
use worth_runtime_world::facade::WorldReleasedOriginal;

pub(super) enum OriginalFate {
    Unpublished(ProductUnpublishedRecoveryHandle),
    Released(WorldReleasedOriginal),
}
pub(super) struct RecoveredIdempotencyEntry {
    binding: Arc<OriginalBinding>,
    fate: OriginalFate,
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
        let original = released_original::read(
            execution,
            self.graph.layout.provider_idempotency(),
            native.semantic_result(),
            |id| {
                native
                    .created_entity(execution, id)
                    .map_err(|_| Denial::Unavailable)
            },
        )?;
        self.admit_original(
            original,
            WorthQueryProductIdempotencyAffinity::from_observation(effects.expected_head()),
            OriginalFate::Unpublished(handle),
        )
    }
    pub(in crate::domain_computation::primary_graph) fn admit_released_idempotency(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &WorldReleasedOriginal,
    ) -> Result<(), Denial> {
        let Some(semantic) = original.native_semantic_result() else {
            return Ok(());
        };
        let binding = released_original::read(
            execution,
            self.graph.layout.provider_idempotency(),
            semantic,
            |id| {
                original
                    .created_entity(execution, id)
                    .map_err(|_| Denial::Unavailable)
            },
        )?;
        self.admit_original(
            binding,
            WorthQueryProductIdempotencyAffinity::from_released(original),
            OriginalFate::Released(original.clone()),
        )
    }
    fn admit_original(
        &self,
        binding: Arc<OriginalBinding>,
        affinity: WorthQueryProductIdempotencyAffinity,
        fate: OriginalFate,
    ) -> Result<(), Denial> {
        if binding.binding.affinity
            != super::super::idempotency::recorded_origin::affinity_text(&affinity)
        {
            return Err(Denial::ForeignAffinity);
        }
        let key = WorthQueryUnpublishedIdempotencyKey::for_original(
            affinity,
            binding.binding.key,
            &binding.binding.intent,
        )?;
        let mut store = self
            .unpublished_idempotency
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(prior) = store.recovered.get(&key) {
            let same = match (&prior.fate, &fate) {
                (OriginalFate::Unpublished(a), OriginalFate::Unpublished(b)) => a == b,
                (OriginalFate::Released(a), OriginalFate::Released(b)) => a.same_original(b),
                (OriginalFate::Unpublished(_), OriginalFate::Released(_))
                | (OriginalFate::Released(_), OriginalFate::Unpublished(_)) => false,
            };
            return if same && prior.binding == binding {
                Ok(())
            } else {
                Err(Denial::Unavailable)
            };
        }
        if store.by_key.contains_key(&key)
            || store
                .maximum_entries
                .is_some_and(|n| store.by_key.len() + store.recovered.len() >= n)
        {
            return Err(Denial::Unavailable);
        }
        store
            .recovered
            .insert(key, RecoveredIdempotencyEntry { binding, fate });
        Ok(())
    }
    pub(in crate::domain_computation) fn prepare_memory_released_binding(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: Option<&worth_relational::facade::history::RelationalCommitReceipt>,
        keyed: bool,
    ) -> Result<
        (
            Option<worth_relational::facade::durability::ObservedRelationalMemoryCommit>,
            Option<Arc<OriginalBinding>>,
        ),
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenialKind,
    >{
        let Some(receipt) = receipt else {
            return Ok((None, None));
        };
        let prepared: Result<_, Denial> = self.graph.with_runtime(execution, |runtime| {
            let native = runtime
                .durability_authority()
                .observe_memory_commit(execution, receipt)
                .map_err(|_| Denial::Unavailable)?;
            let binding = if keyed {
                Some(released_original::read(
                    execution,
                    self.graph.layout.provider_idempotency(),
                    native.semantic_result().semantic(),
                    |id| {
                        native
                            .created_entity(execution, id)
                            .map_err(|_| Denial::Unavailable)
                    },
                )?)
            } else {
                None
            };
            Ok((Some(native), binding))
        });
        prepared.map_err(|cause| crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial::from_provider(cause).kind())
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
            match binding.match_recorded_intent(&entry.binding.binding.intent)? {
                WorthQueryRecordedIntentMatch::Same => match &entry.fate {
                    OriginalFate::Unpublished(handle) => {
                        WorthQueryProviderIdempotencyResolution::RecoveredUnpublished {
                            handle: handle.clone(),
                        }
                    }
                    OriginalFate::Released(original) => {
                        WorthQueryProviderIdempotencyResolution::ReleasedOriginal(
                            WorthQueryReleasedApplicationEffect::new(
                                entry.binding.clone(),
                                original.clone(),
                            ),
                        )
                    }
                },
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
            .retain(|_, entry| match &entry.fate {
                OriginalFate::Unpublished(current) => current != handle,
                OriginalFate::Released(_) => true,
            });
    }
    pub(in crate::domain_computation) fn finish_recovered_release(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
        original: WorldReleasedOriginal,
    ) {
        let mut store = self.store.lock().unwrap_or_else(|e| e.into_inner());
        if original.native_semantic_result().is_none() {
            return;
        }
        let entry = store
            .recovered
            .values_mut()
            .find(|entry| match &entry.fate {
                OriginalFate::Unpublished(current) => current == handle,
                OriginalFate::Released(_) => false,
            })
            .expect("D8 admitted the exact original key before serving release");
        entry.fate = OriginalFate::Released(original);
    }
}
