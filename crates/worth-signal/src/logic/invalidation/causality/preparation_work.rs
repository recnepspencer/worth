//! Concrete cause payload copying and scope-comparison work admission.
use crate::data::error::SignalError;
use crate::data::output::PartitionSubscription;
use crate::data::proof::invalidation::{
    binding::ResolvedDependencyCause, output_commit::ProducedAspectDelta,
};
use crate::data::request_preparation::SignalPreparationBudget;
use crate::logic::evaluation::EvaluationWork;

pub(super) fn scope_comparison(scope: Option<&PartitionSubscription>) -> Option<usize> {
    scope_bytes(scope)?.checked_mul(2)?.checked_add(16)
}

pub(super) fn scope_bytes(scope: Option<&PartitionSubscription>) -> Option<usize> {
    match scope {
        None => Some(0),
        Some(scope) => Some(scope.path().total_segment_bytes()),
    }
}

pub(super) fn admit_scope_copy(
    scope: Option<&PartitionSubscription>,
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(), SignalError> {
    let Some(scope) = scope else {
        return work.reserve(Some(0));
    };
    work.reserve(
        scope_bytes(Some(scope))
            .and_then(|bytes| {
                scope
                    .path()
                    .depth()
                    .checked_mul(std::mem::size_of::<String>())
                    .and_then(|slots| slots.checked_add(bytes))
            })
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<PartitionSubscription>())),
    )
}

pub(super) fn admit_scopes_copy(
    scopes: &[PartitionSubscription],
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(), SignalError> {
    work.reserve(Some(scopes.len()))?;
    for scope in scopes {
        admit_scope_copy(Some(scope), work)?;
    }
    Ok(())
}

pub(super) fn admit_causes_copy(
    causes: &[ResolvedDependencyCause],
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(), SignalError> {
    work.reserve(
        causes
            .len()
            .checked_mul(std::mem::size_of::<ResolvedDependencyCause>()),
    )?;
    for cause in causes {
        // The key and binding own distinct scope strings; neither copy is free.
        admit_scope_copy(cause.key.edge_scope.as_ref(), work)?;
        admit_scope_copy(cause.binding_axes.edge_scope.as_ref(), work)?;
        admit_scopes_copy(cause.changed_scopes.as_slice(), work)?;
    }
    Ok(())
}

pub(super) fn claim_causes_copy(
    causes: &[ResolvedDependencyCause],
    capacity: usize,
    mut budget: Option<&mut SignalPreparationBudget>,
) -> Result<(), SignalError> {
    if let Some(budget) = budget.as_deref_mut() {
        budget.claim_vec::<ResolvedDependencyCause>(capacity)?;
    }
    for cause in causes {
        claim_scope_copy(cause.key.edge_scope.as_ref(), budget.as_deref_mut())?;
        claim_scope_copy(
            cause.binding_axes.edge_scope.as_ref(),
            budget.as_deref_mut(),
        )?;
        if !cause.changed_scopes.is_empty() {
            if let Some(budget) = budget.as_deref_mut() {
                budget.claim_vec::<PartitionSubscription>(cause.changed_scopes.len())?;
            }
        }
        for scope in cause.changed_scopes.as_slice() {
            claim_scope_copy(Some(scope), budget.as_deref_mut())?;
        }
    }
    Ok(())
}

pub(super) fn claim_cause_normalization_and_cache(
    causes: &[ResolvedDependencyCause],
    mut budget: Option<&mut SignalPreparationBudget>,
) -> Result<(), SignalError> {
    if let Some(budget) = budget.as_deref_mut() {
        // The canonical normalizer returns a zero- or one-cause input Vec
        // directly, without an index or a second cause backing.
        if causes.len() > 1 {
            budget.claim_vec::<(usize, ResolvedDependencyCause)>(causes.len())?;
            budget.claim_vec::<ResolvedDependencyCause>(causes.len())?;
        }
        let scope_count = causes
            .iter()
            .try_fold(0usize, |count, cause| {
                count.checked_add(cause.changed_scopes.len())
            })
            .ok_or_else(|| SignalError::invalid_input("cause scope count overflow"))?;
        if scope_count != 0 {
            budget
                .claim_vec::<(crate::data::aspect::Aspect, PartitionSubscription)>(scope_count)?;
        }
    }
    for cause in causes {
        for scope in cause.changed_scopes.as_slice() {
            claim_scope_copy(Some(scope), budget.as_deref_mut())?;
        }
    }
    Ok(())
}

pub(super) fn claim_scope_copy(
    scope: Option<&PartitionSubscription>,
    budget: Option<&mut SignalPreparationBudget>,
) -> Result<(), SignalError> {
    if let (Some(scope), Some(budget)) = (scope, budget) {
        budget.claim_vec::<String>(scope.path().depth())?;
        budget.claim_vec::<u8>(scope.path().total_segment_bytes())?;
    }
    Ok(())
}

pub(super) fn claim_delta_copy(
    delta: &ProducedAspectDelta,
    mut budget: Option<&mut SignalPreparationBudget>,
) -> Result<(), SignalError> {
    if let Some(budget) = budget.as_deref_mut() {
        budget.claim_vec::<crate::data::proof::invalidation::output_commit::ProducedAspectChange>(
            delta.changes.as_slice().len(),
        )?;
    }
    for change in delta.changes.as_slice() {
        if !change.changed_scopes.is_empty() {
            if let Some(budget) = budget.as_deref_mut() {
                budget.claim_vec::<PartitionSubscription>(change.changed_scopes.len())?;
            }
        }
        for scope in change.changed_scopes.as_slice() {
            claim_scope_copy(Some(scope), budget.as_deref_mut())?;
        }
    }
    Ok(())
}

pub(crate) fn admit_delta_copy(
    delta: &ProducedAspectDelta,
    work: &mut EvaluationWork<'_, '_>,
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
    work: &mut EvaluationWork<'_, '_>,
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
