use super::{preparation_work, scope_normalization};
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::output::{scopes_overlap, PartitionSubscription};
use crate::data::proof::invalidation::binding::{
    DependencyRevision, OutputCommitOrdinal, ResolvedDependencyCause,
};
use crate::data::proof::invalidation::output_commit::ProducedAspectChange;
use crate::data::proof::PartitionScopeSet;
use crate::logic::evaluation::EvaluationWork;

#[derive(Clone, Copy)]
pub(crate) struct CauseAdmissionContext {
    pub(crate) graph_instance: u64,
    pub(crate) consumer: NodeId,
    pub(crate) revision: DependencyRevision,
    pub(crate) producer: NodeId,
    pub(crate) output_commit_ordinal: OutputCommitOrdinal,
}

pub(crate) fn changed_scopes_for_edge<'a>(
    change: &ProducedAspectChange,
    edge_scope: Option<&'a PartitionSubscription>,
    work: &mut EvaluationWork<'_>,
) -> Result<Option<&'a [PartitionSubscription]>, SignalError> {
    let Some(edge_scope) = edge_scope else {
        return Ok(Some(&[]));
    };
    work.reserve(
        preparation_work::scope_comparison(Some(edge_scope))
            .and_then(|cost| cost.checked_mul(change.changed_scopes.len())),
    )?;
    Ok((change.changed_scopes.is_empty()
        || change
            .changed_scopes
            .iter()
            .any(|changed| scopes_overlap(changed, edge_scope)))
    .then(|| std::slice::from_ref(edge_scope)))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn reconcile_edge_cause(
    causes: &mut Vec<ResolvedDependencyCause>,
    context: CauseAdmissionContext,
    aspect: crate::data::aspect::Aspect,
    edge_scope: Option<&PartitionSubscription>,
    cached_version: u64,
    committed_version: u64,
    changed_scopes: &[PartitionSubscription],
    meaningful: bool,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(
        preparation_work::scope_comparison(edge_scope)
            .and_then(|cost| cost.checked_add(4))
            .and_then(|cost| cost.checked_mul(causes.len())),
    )?;
    let existing = causes.iter().position(|cause| {
        cause.key.producer == context.producer
            && cause.key.aspect == aspect
            && cause.key.edge_scope.as_ref() == edge_scope
    });
    let prior_scopes = existing.map(|index| causes.remove(index).changed_scopes);
    let Some(reconciled_scopes) =
        reconcile_changed_scopes(prior_scopes.as_ref(), changed_scopes, meaningful, work)?
    else {
        return Ok(());
    };
    work.reserve(causes.len().checked_add(32))?;
    // Constructor owns a scope in both key and binding.
    preparation_work::admit_scope_copy(edge_scope, work)?;
    preparation_work::admit_scope_copy(edge_scope, work)?;
    causes.push(ResolvedDependencyCause::new(
        context.graph_instance,
        context.consumer,
        context.revision,
        context.producer,
        aspect,
        edge_scope.cloned(),
        cached_version,
        context.output_commit_ordinal,
        committed_version,
        reconciled_scopes,
    ));
    Ok(())
}

fn reconcile_changed_scopes(
    prior: Option<&PartitionScopeSet>,
    touched: &[PartitionSubscription],
    meaningful: bool,
    work: &mut EvaluationWork<'_>,
) -> Result<Option<PartitionScopeSet>, SignalError> {
    if touched.is_empty() {
        return Ok(meaningful.then(PartitionScopeSet::default));
    }
    if prior.is_some_and(PartitionScopeSet::is_empty) {
        return Ok(Some(PartitionScopeSet::default()));
    }
    let previous = prior.map_or(&[][..], PartitionScopeSet::as_slice);
    work.reserve(previous.len().checked_add(touched.len()))?;
    for scope in previous {
        work.reserve(
            preparation_work::scope_comparison(Some(scope))
                .and_then(|cost| cost.checked_mul(touched.len())),
        )?;
    }
    preparation_work::admit_scopes_copy(previous, work)?;
    if meaningful {
        preparation_work::admit_scopes_copy(touched, work)?;
    }
    let count = previous
        .len()
        .checked_add(if meaningful { touched.len() } else { 0 });
    work.reserve(
        count
            .and_then(|count| count.checked_mul(std::mem::size_of::<PartitionSubscription>()))
            .filter(|bytes| *bytes <= isize::MAX as usize),
    )?;
    let mut scopes = Vec::with_capacity(count.expect("admitted scope count"));
    scopes.extend(
        previous
            .iter()
            .filter(|scope| {
                !touched
                    .iter()
                    .any(|changed| scopes_overlap(*scope, changed))
            })
            .cloned(),
    );
    if meaningful {
        scopes.extend(touched.iter().cloned());
    }
    if scopes.is_empty() {
        return Ok(None);
    }
    scope_normalization::normalize(scopes, work).map(Some)
}
