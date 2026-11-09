//! Co-committed outbox data read from the original Native after-image.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph::provider) fn original_dispatch_outbox(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        publication: &super::super::OriginalPublication,
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
        let mut rows = semantic
            .created_entities()
            .iter()
            .filter(|created| created.key.kind == layout.entity_kind);
        let Some(created) = rows.next() else {
            return Ok(None);
        };
        if rows.next().is_some() {
            return Err(Denial::AmbiguousCorrelation);
        }
        let fields = required_fields(layout)?;
        let aspect = layout.correlation_locator.aspect().aspect_key().clone();
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
        Ok(Some((RecordRef::Entity(created.assigned), record)))
    }
}
