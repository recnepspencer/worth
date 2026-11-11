//! Co-committed outbox data read from the original Native after-image.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph::provider) fn original_dispatch_outbox(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        publication: &super::super::OriginalPublication,
        outcome: crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcomeIdentity,
    ) -> Result<Option<(RecordRef, WorthQueryDispatchOutboxRestoredFields)>, Denial> {
        execution.consult().map_err(|cause| {
            Denial::ReadExecution(
                super::super::relational_execution_denial::relational_execution_kind(
                    cause.into(),
                    None,
                ),
            )
        })?;
        let layout = self.graph.layout.provider_dispatch_outbox();
        let semantic = publication.semantic_result();
        let outcome_field = layout
            .outcome_identity_locator
            .field_path()
            .fields()
            .first()
            .ok_or(Denial::Malformed)?;
        let mut selected = None;
        for created in semantic
            .created_entities()
            .iter()
            .filter(|created| created.key.kind == layout.entity_kind)
        {
            let row = publication
                .created_entity(execution, created.assigned)
                .map_err(|cause| {
                    if let worth_relational::facade::durability::RelationalAssociatedEffectRefusal::Recovery(
                        worth_relational::facade::durability::RelationalRecoveryStop::Request(cause)
                    ) = cause {
                        Denial::ReadExecution(super::super::relational_execution_denial::relational_execution_kind(cause, None))
                    } else { Denial::ExactCommitUnavailable }
                })?
                .ok_or(Denial::RecordMismatch)?;
            if row.kind_id() != layout.entity_kind {
                return Err(Denial::WrongRecordKind);
            }
            let Some(AspectValue::UInt64(recorded_outcome)) = row.field(
                layout.outcome_identity_locator.aspect().aspect_key(),
                outcome_field,
            ) else {
                return Err(Denial::Malformed);
            };
            if *recorded_outcome != outcome.get() {
                continue;
            }
            if selected.replace((created.assigned, row)).is_some() {
                return Err(Denial::AmbiguousCorrelation);
            }
        }
        let Some((entity, row)) = selected else {
            return Ok(None);
        };
        let fields = required_fields(layout)?;
        let aspect = layout.correlation_locator.aspect().aspect_key().clone();
        let values = fields
            .iter()
            .map(|field| row.field(&aspect, field).cloned())
            .collect::<Option<Vec<_>>>()
            .ok_or(Denial::Malformed)?;
        let record = decode_fields(values)?;
        if u64::try_from(record.payload.len()).map_err(|_| Denial::Malformed)?
            > record.maximum_payload_bytes
        {
            return Err(Denial::Malformed);
        }
        Ok(Some((RecordRef::Entity(entity), record)))
    }
}
