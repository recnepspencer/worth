//! Request-aware exact-root observation of the principal mapping.
use super::*;

pub(in crate::domain_computation::primary_graph) fn observe_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    mapping_id: EntityId,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
    binding: &str,
) -> Result<WorthQueryPrincipalMappingObservation, WorthQueryPrincipalResolutionDenial> {
    execution.consult().map_err(|stop| {
        super::super::resolution_denial::query_read_resolution_denial(
            worth_relational::facade::runtime::QueryReadExecutionStop::Request(stop),
            binding,
        )
    })?;
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot, execution)
        .ok_or_else(|| stale_proof_denial(binding))?;
    let scope = mapping_projection_scope(layout);
    let identity_revision = view
        .entity_field_revision(
            execution,
            mapping_id,
            &authoritative(&layout.identity_locator),
        )
        .map_err(|cause| super::super::resolution_denial::relational_read_denial(cause, binding))?
        .ok_or_else(|| stale_proof_denial(binding))?;
    let status_revision = view
        .entity_field_revision(
            execution,
            mapping_id,
            &authoritative(&layout.status_locator),
        )
        .map_err(|cause| super::super::resolution_denial::relational_read_denial(cause, binding))?
        .ok_or_else(|| stale_proof_denial(binding))?;
    view.entity_record_with_projection_scope(execution, mapping_id, scope, |record| {
        if record.kind_id() != layout.mapping_kind
            || record.lifecycle() != RecordLifecycleState::Live
        {
            return None;
        }
        let identity = projected_field(record, &layout.identity_locator)?.clone();
        let AspectValue::Bool(enabled) = projected_field(record, &layout.status_locator)? else {
            return None;
        };
        Some(WorthQueryPrincipalMappingObservation {
            entity_id: record.entity_id(),
            kind_id: record.kind_id(),
            identity,
            enabled: *enabled,
            identity_revision,
            status_revision,
        })
    })
    .map_err(|cause| super::super::resolution_denial::relational_read_denial(cause, binding))?
    .ok_or_else(|| stale_proof_denial(binding))
}
