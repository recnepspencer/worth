//! The published operation contract is data, never a reconstructed binding or permit.
use super::super::observed_optional_text;
use super::{
    invalid_connection, CompiledWorkflowConnectionKind, EntityId,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
    WorthQueryWorkflowLayout,
};
pub(super) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<CompiledWorkflowConnectionKind, WorthQueryApplicationAttemptDenial> {
    let action = &layout.connection.back_action;
    let mut read = |locator| {
        observed_optional_text(
            execution,
            runtime,
            snapshot,
            entity,
            layout.connection.entity_kind,
            locator,
            facts,
        )?
        .filter(|value| !value.is_empty())
        .ok_or_else(invalid_connection)
    };
    let operation = read(&action.operation)?;
    let input_type = read(&action.input_type)?;
    let binding = read(&action.binding)?;
    let encoded = read(&action.requirements)?;
    let requirements = worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements::from_persistence_identity(&encoded)
        .filter(|requirements| requirements.approval().is_none()).ok_or_else(invalid_connection)?;
    Ok(CompiledWorkflowConnectionKind::BackAction {
        operation,
        input_type,
        binding,
        requirements,
    })
}
