//! Concrete cause payload copying and scope-comparison work admission.
use crate::data::error::SignalError;
use crate::data::output::PartitionSubscription;
use crate::data::proof::invalidation::{
    binding::ResolvedDependencyCause, output_commit::ProducedAspectDelta,
};
use crate::logic::evaluation::EvaluationWork;

pub(super) fn scope_comparison(scope: Option<&PartitionSubscription>) -> Option<usize> {
    scope_bytes(scope)?.checked_mul(2)?.checked_add(16)
}

pub(super) fn scope_bytes(scope: Option<&PartitionSubscription>) -> Option<usize> {
    match scope {
        None => Some(0),
        Some(scope) => scope
            .partition
            .0
            .len()
            .checked_add(scope.detail.as_ref().map_or(0, String::len)),
    }
}

pub(super) fn admit_scope_copy(
    scope: Option<&PartitionSubscription>,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(scope_bytes(scope).and_then(|bytes| bytes.checked_add(4)))
}

pub(super) fn admit_scopes_copy(
    scopes: &[PartitionSubscription],
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(Some(scopes.len()))?;
    for scope in scopes {
        admit_scope_copy(Some(scope), work)?;
    }
    Ok(())
}

pub(super) fn admit_causes_copy(
    causes: &[ResolvedDependencyCause],
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(causes.len().checked_mul(32))?;
    for cause in causes {
        // The key and binding own distinct scope strings; neither copy is free.
        admit_scope_copy(cause.key.edge_scope.as_ref(), work)?;
        admit_scope_copy(cause.binding_axes.edge_scope.as_ref(), work)?;
        admit_scopes_copy(cause.changed_scopes.as_slice(), work)?;
    }
    Ok(())
}

pub(crate) fn admit_delta_copy(
    delta: &ProducedAspectDelta,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(
        delta
            .changes
            .as_slice()
            .len()
            .checked_mul(8)
            .and_then(|n| n.checked_add(crate::data::aspect::MAX_ASPECTS)),
    )?;
    for change in delta.changes.as_slice() {
        admit_scopes_copy(change.changed_scopes.as_slice(), work)?;
    }
    Ok(())
}

/// Equality examines at most the query's scopes and bytes from each operand.
/// Different lengths stop equality before any unmatched payload is visited.
pub(super) fn admit_delta_comparison(
    delta: &ProducedAspectDelta,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(
        delta
            .changes
            .as_slice()
            .len()
            .checked_mul(8)
            .and_then(|n| n.checked_add(crate::data::aspect::MAX_ASPECTS)),
    )?;
    for change in delta.changes.as_slice() {
        work.reserve(Some(change.changed_scopes.as_slice().len()))?;
        for scope in change.changed_scopes.as_slice() {
            work.reserve(scope_comparison(Some(scope)))?;
        }
    }
    Ok(())
}
