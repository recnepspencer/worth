//! Classify actual original completion rows before attempting keyed idempotency admission.
use super::read::{
    completion_locators, decode_completion_values, WorthQueryCanonicalCompletionRow,
    WorthQueryInboundCompletionReadDenial as Denial,
};
use super::*;
use worth_relational::facade::transactions::RelationalCommitSemanticResult;
use worth_runtime_world::facade::{WorldRecoveredUnpublishedEffects, WorldReleasedOriginal};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn recovered_inbound_completion_row(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effects: &WorldRecoveredUnpublishedEffects,
    ) -> Result<
        Option<(
            worth_relational::facade::identity::EntityId,
            WorthQueryCanonicalCompletionRow,
        )>,
        Denial,
    > {
        let Some(native) = effects.native_effect() else {
            return Ok(None);
        };
        let Some(entity) = self.completion_entity(native.semantic_result())? else {
            return Ok(None);
        };
        let row = native
            .created_entity(execution, entity)
            .map_err(super::source_read::native_read_denial)?
            .ok_or(Denial::Malformed)?;
        self.decode_original_completion(execution, &row, native.semantic_result().receipt().clone())
            .map(|row| Some((entity, row)))
    }

    pub(in crate::domain_computation::primary_graph) fn released_inbound_completion_row(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &WorldReleasedOriginal,
    ) -> Result<Option<WorthQueryCanonicalCompletionRow>, Denial> {
        let Some(semantic) = original.native_semantic_result() else {
            return Ok(None);
        };
        let Some(entity) = self.completion_entity(semantic)? else {
            return Ok(None);
        };
        let row = original
            .created_entity(execution, entity)
            .map_err(|_| Denial::CommitUnavailable)?
            .ok_or(Denial::Malformed)?;
        self.decode_original_completion(execution, &row, semantic.receipt().clone())
            .map(Some)
    }

    fn completion_entity(
        &self,
        semantic: &RelationalCommitSemanticResult,
    ) -> Result<Option<worth_relational::facade::identity::EntityId>, Denial> {
        let mut completion = None;
        for created in semantic.created_entities() {
            if created.key.kind == self.graph.layout.provider_inbound_completion().kind
                && completion.replace(created.assigned).is_some()
            {
                return Err(Denial::AmbiguousCorrelation);
            }
        }
        if completion.is_some() && semantic.created_entities().len() != 1 {
            return Err(Denial::Malformed);
        }
        Ok(completion)
    }

    fn decode_original_completion(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        row: &worth_relational::facade::durability::ReadmittedRelationalCreatedEntity<'_>,
        receipt: worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<WorthQueryCanonicalCompletionRow, Denial> {
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
            decode_completion_values(runtime, &values, receipt)
        })
    }
}

impl WorthQueryCanonicalCompletionRow {
    pub(in crate::domain_computation::primary_graph) fn matches_original(
        &self,
        owner: &super::super::WorthQueryCommittedDispatchOutboxObservation,
    ) -> bool {
        let record = owner.record();
        let Some(contract) = record.inbound() else {
            return false;
        };
        let RecordRef::Entity(entity) = owner.record_ref() else {
            return false;
        };
        self.correlation == *record.correlation()
            && record.operation_slot() == Some(self.operation.as_str())
            && self.family == record.correlation_family().as_str()
            && self.source == contract.source_identity()
            && self.protocol_identity == *record.protocol_identity()
            && self.protocol_version == record.protocol_version()
            && self.payload == record.payload()
            && self.original_commit == *owner.commit_reference()
            && self.original_incarnation_ordinal
                == owner
                    .committed_product_publication()
                    .product_incarnation()
                    .ordinal()
            && self.original_entity
                == format!(
                    "{}:{}:{}",
                    entity.partition_value_u64(),
                    entity.local_slot_value(),
                    entity.generation_value()
                )
    }
}
