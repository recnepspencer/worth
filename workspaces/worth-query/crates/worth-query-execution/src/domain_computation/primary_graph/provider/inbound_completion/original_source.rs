//! Original outbox meaning comes from owner artifacts, never an ordinary live lease.
use super::read::{
    WorthQueryCanonicalCompletionRow, WorthQueryInboundCompletionReadDenial as Denial,
};
use super::source_read::{native_read_denial, world_read_denial};
use super::*;
use crate::domain_computation::primary_graph::provider::OriginalPublication;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchIncarnation, RuntimeWorldObservationPort,
};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn completion_original_outbox(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        completed: &WorthQueryCanonicalCompletionRow,
        world: &CompositeCommitIdentity,
        incarnation: ProductBranchIncarnation,
        observation: &RuntimeWorldObservationPort,
        maximum_created_records: std::num::NonZeroUsize,
    ) -> Result<
        crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
        Denial,
    > {
        self.graph.with_runtime(execution, |runtime| {
            let source = original_publication(
                runtime,
                execution,
                &completed.original_commit,
                world,
                observation,
            )?;
            if !matches_world(&source, world, incarnation) {
                return Err(Denial::Malformed);
            }
            self.outbox_from_original_bindings(
                execution,
                &source,
                completed,
                maximum_created_records,
            )
        })
    }

    fn outbox_from_original_bindings(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &OriginalPublication,
        completed: &WorthQueryCanonicalCompletionRow,
        maximum_created_records: std::num::NonZeroUsize,
    ) -> Result<
        crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
        Denial,
    > {
        let created = source.semantic_result().created_entities();
        if created.len() > maximum_created_records.get() {
            return Err(Denial::ReconstructionWorkExhausted);
        }
        let layout = self.graph.layout.provider_idempotency();
        let expected_affinity =
            super::super::idempotency::recorded_origin::original_affinity_text(source);
        let mut selected = None;
        for entity in created {
            if entity.key.kind != layout.entity_kind {
                continue;
            }
            let row = source
                .created_entity(execution, entity.assigned)
                .map_err(native_read_denial)?
                .ok_or(Denial::Malformed)?;
            let binding = super::super::idempotency::snapshot_resolution::original_binding::read(
                &row, layout,
            )
            .map_err(binding_read_denial)?;
            if binding.affinity != expected_affinity {
                return Err(Denial::Malformed);
            }
            let Some((record_ref, fields)) = self
                .original_dispatch_outbox(execution, source, binding.outcome)
                .map_err(Denial::SourceRead)?
            else {
                continue;
            };
            let RecordRef::Entity(original_entity) = record_ref else {
                return Err(Denial::Malformed);
            };
            if fields.correlation != completed.correlation
                || format!(
                    "{}:{}:{}",
                    original_entity.partition_value_u64(),
                    original_entity.local_slot_value(),
                    original_entity.generation_value()
                ) != completed.original_entity
            {
                continue;
            }
            let rebound = super::super::WorthQueryRecoveredDispatchOutbox::from_source_row(
                record_ref,
                fields,
                &binding.intent,
                &self.installed_operations,
            )
            .readmitted_record()
            .map_err(Denial::SourceRead)?;
            if selected.replace(rebound).is_some() {
                return Err(Denial::AmbiguousCorrelation);
            }
        }
        selected.ok_or(Denial::Malformed)
    }
}

fn original_publication(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    receipt: &worth_relational::facade::history::RelationalCommitReceipt,
    world: &CompositeCommitIdentity,
    observation: &RuntimeWorldObservationPort,
) -> Result<OriginalPublication, Denial> {
    let authority = runtime.durability_authority();
    if world.owner_identity().durable().is_none() {
        let native = authority
            .observe_memory_commit(execution, receipt)
            .map_err(native_read_denial)?;
        return observation
            .observe_memory_original_publication(execution, native)
            .map(OriginalPublication::Memory)
            .map_err(world_read_denial);
    }
    let mut exact = None;
    for subject in authority
        .discover_all_associated_effects(execution)
        .map_err(native_read_denial)?
    {
        if subject.receipt() == receipt && exact.replace(subject).is_some() {
            return Err(Denial::AmbiguousCorrelation);
        }
    }
    let native = authority
        .readmit_associated_effect(execution, &exact.ok_or(Denial::CommitUnavailable)?)
        .map_err(native_read_denial)?;
    observation
        .readmitted_native_publication(execution, native)
        .map(OriginalPublication::Home)
        .map_err(world_read_denial)
}

fn matches_world(
    source: &OriginalPublication,
    world: &CompositeCommitIdentity,
    incarnation: ProductBranchIncarnation,
) -> bool {
    match source {
        OriginalPublication::Home(source) => {
            source.after().selected_commit() == world
                && source.after().lifecycle_incarnation() == incarnation
        }
        // The actual World observation service admits the current receiver
        // against the exact Native artifact; these are original DATA coordinates.
        OriginalPublication::Memory(source) => {
            source.after().selected_commit() == world.ordinal()
                && source.after().incarnation() == incarnation.ordinal()
        }
    }
}

fn binding_read_denial(
    cause: super::super::WorthQueryProviderIdempotencyResolutionDenial,
) -> Denial {
    Denial::SourceRead(
        super::super::committed_dispatch_outbox::recovered::idempotency_denial(cause),
    )
}
