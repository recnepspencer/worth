//! Retain each reconstructed branch transition under the original recovery request.
use super::*;
pub(super) fn copy_transition(
    entry: &crate::durability::source_initial_inputs::normalized::TransitionOriginalRecord,
    graph: &OriginalSchemaGraph,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RetainedBranchTransition, RelationalRecoveryStop> {
    use crate::branch::{PositionedBranchTransition, RelationalBranchTransition};
    let dynamic = match &entry.transition {
        RelationalBranchTransition::Fork {
            target,
            source,
            source_head,
            ..
        }
        | RelationalBranchTransition::RetainedFork {
            target,
            source,
            source_head,
            ..
        } => target.0.len().checked_add(source.0.len()).and_then(|n| {
            source_head
                .parent_commit_ids()
                .len()
                .checked_mul(8)
                .and_then(|parents| n.checked_add(parents))
        }),
        RelationalBranchTransition::Archive { branch, .. }
        | RelationalBranchTransition::DeletionRequested { branch, .. }
        | RelationalBranchTransition::Delete { branch, .. } => Some(branch.0.len()),
    }
    .ok_or_else(|| corrupt("transition source backing overflow"))?;
    let bytes = arc_size::<PositionedBranchTransition>()?
        .checked_add(arc_size::<worth_execution::ExecutionMemoryReservation>()?)
        .and_then(|n| n.checked_add(dynamic as u64))
        .ok_or_else(|| corrupt("transition source control overflow"))?;
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?
        .map(Arc::new);
    let positioned =
        PositionedBranchTransition::admit(entry.sequence, entry.follows, entry.transition.clone())
            .with_schema_carrier(Some(entry.schema))
            .with_source_memory(memory);
    Ok(RetainedBranchTransition {
        positioned: Arc::new(positioned),
        schema: graph.get(entry.schema)?,
    })
}
fn arc_size<T>() -> Result<u64, RelationalRecoveryStop> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map(|(layout, _)| layout.pad_to_align().size() as u64)
        .map_err(|_| corrupt("transition source layout overflow").into())
}
