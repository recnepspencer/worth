//! Exact completion creation rows from actual Native and World readmission.
use super::read::{
    completion_locators, decode_completion_values, WorthQueryCanonicalCompletionRow,
    WorthQueryInboundCompletionReadDenial as Denial,
};
use super::*;
use worth_runtime_world::facade::{RuntimeWorldObservationPort, WorldReadmittedPublication};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn completion_source_from_subject(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        subject: &worth_relational::facade::durability::RelationalAssociatedEffectSubject,
        observation: &RuntimeWorldObservationPort,
    ) -> Result<WorldReadmittedPublication, Denial> {
        self.graph.with_runtime(execution, |runtime| {
            let native = runtime
                .durability_authority()
                .readmit_associated_effect(execution, subject)
                .map_err(native_read_denial)?;
            observation
                .readmitted_native_publication(execution, native)
                .map_err(world_read_denial)
        })
    }
    pub(in crate::domain_computation::primary_graph) fn completion_row_from_readmitted_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &WorldReadmittedPublication,
        entity: worth_relational::facade::identity::EntityId,
    ) -> Result<WorthQueryCanonicalCompletionRow, Denial> {
        let row = original
            .created_entity(execution, entity)
            .map_err(native_read_denial)?
            .ok_or(Denial::Malformed)?;
        let layout = self.graph.layout.provider_inbound_completion();
        if row.kind_id() != layout.kind {
            return Err(Denial::Malformed);
        }
        let aspect = layout.correlation.aspect().aspect_key();
        let values = completion_locators(layout)
            .iter()
            .map(|locator| {
                let key = super::read::field(locator)?;
                row.field(aspect, &key).cloned().ok_or(Denial::Malformed)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.graph.with_runtime(execution, |runtime| {
            decode_completion_values(runtime, &values, original.native_receipt().clone())
        })
    }

    pub(in crate::domain_computation::primary_graph) fn readmit_inbound_completion_sources(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &RuntimeWorldObservationPort,
        unpublished_native: &std::collections::BTreeSet<
            worth_relational::facade::history::CommitId,
        >,
    ) -> Result<
        Vec<(
            WorldReadmittedPublication,
            worth_relational::facade::identity::EntityId,
        )>,
        Denial,
    > {
        self.graph.with_runtime(execution, |runtime| {
            let authority = runtime.durability_authority();
            let subjects = authority
                .discover_all_associated_effects(execution)
                .map_err(native_read_denial)?;
            let mut completions = Vec::new();
            for subject in subjects {
                if unpublished_native.contains(&subject.receipt().commit_id) {
                    continue;
                }
                let native = authority
                    .readmit_associated_effect(execution, &subject)
                    .map_err(native_read_denial)?;
                let mut entity = None;
                for created in native.semantic_result().created_entities() {
                    if created.key.kind == self.graph.layout.provider_inbound_completion().kind
                        && entity.replace(created.assigned).is_some()
                    {
                        return Err(Denial::AmbiguousCorrelation);
                    }
                }
                if let Some(entity) = entity {
                    let world = observation
                        .readmitted_native_publication(execution, native)
                        .map_err(world_read_denial)?;
                    completions.push((world, entity));
                }
            }
            Ok(completions)
        })
    }
}

pub(super) fn native_read_denial(
    cause: worth_relational::facade::durability::RelationalAssociatedEffectRefusal,
) -> Denial {
    Denial::SourceRead(super::super::committed_dispatch_outbox::recovered::native_denial(cause))
}
pub(super) fn world_read_denial(
    cause: worth_runtime_world::facade::RuntimeWorldServiceDenial<
        worth_runtime_world::facade::WorldReadmittedPublicationDenial,
    >,
) -> Denial {
    Denial::SourceRead(super::super::committed_dispatch_outbox::recovered::world_denial(cause))
}
