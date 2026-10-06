use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::output::PartitionSubscription;
use crate::data::proof::DedupedNodeBatch;
use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};
use crate::logic::prepared::{PreparedDependencyCapture, PreparedDependencyEdge};
use worth_execution::MapKernelContext;

#[derive(Debug, Clone, Default)]
pub(crate) struct MaybeStalePreview {
    pub(crate) unchanged: bool,
    pub(crate) requires_upstream_evaluation: Vec<NodeId>,
}

pub(crate) fn capture_current_dependencies_without_refresh(
    graph: &SignalGraph,
    node: NodeId,
    work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<PreparedDependencyCapture, SignalError> {
    let dependencies = graph.current_runtime_dependencies_of(node)?;
    let scope_bytes = dependencies
        .iter()
        .try_fold(0_u64, |bytes, edge| {
            let Some(scope) = edge.scope_ref() else {
                return Some(bytes);
            };
            let fixed = scope
                .path()
                .depth()
                .checked_mul(std::mem::size_of::<String>())?;
            bytes
                .checked_add(u64::try_from(fixed).ok()?)?
                .checked_add(u64::try_from(scope.path().checked_segment_bytes()?).ok()?)
        })
        .ok_or_else(|| SignalError::invalid_input("dependency capture memory overflow"))?;
    super::super::precompute::work::checkpoint(
        work,
        dependencies
            .len()
            .saturating_mul(dependencies.len().saturating_add(1))
            .saturating_add(usize::try_from(scope_bytes).unwrap_or(usize::MAX)),
    )?;
    if let Some(budget) = preparation {
        budget.claim_vec::<PreparedDependencyEdge>(dependencies.len())?;
        budget.claim(scope_bytes)?;
    }
    let mut capture = PreparedDependencyCapture::with_capacity(dependencies.len());
    for dependency in dependencies {
        capture.record(
            dependency.source(),
            dependency.aspect(),
            dependency.scope_ref().cloned(),
        );
    }
    Ok(capture.into_sorted_unique())
}

pub(crate) fn preview_maybe_stale(
    graph: &SignalGraph,
    node: NodeId,
    resolver: &mut impl ComparatorPolicyResolver,
    mut work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<MaybeStalePreview, SignalError> {
    let snapshot = graph.get_dep_snapshot(node)?;
    let comparator =
        resolver.policy_for_node(node, graph.node_eval_config(node)?.comparator.as_ref());
    let pending = graph.node_pending_revalidation(node)?;
    let pending_count = pending.map_or(0, |pending| pending.unresolved_producers().len());
    super::super::precompute::work::checkpoint(work.as_deref_mut(), pending_count)?;
    let capacity = pending_count
        .checked_add(snapshot.entries().len())
        .ok_or_else(|| SignalError::invalid_input("stale preview capacity overflow"))?;
    preparation_budget::claim_vec::<NodeId>(preparation, capacity)?;
    let mut requires_upstream_evaluation = Vec::with_capacity(capacity);
    if let Some(pending) = pending {
        requires_upstream_evaluation.extend_from_slice(pending.unresolved_producers());
    }
    let mut meaningful_change_detected = pending
        .as_ref()
        .is_some_and(|pending| pending.requires_structural_recompute());

    for snapshot_entry in snapshot.entries() {
        let source_scopes = graph
            .node_runtime_artifact_hot(snapshot_entry.source)
            .ok()
            .flatten()
            .map_or(0, |hot| hot.changed_scopes.as_slice().len());
        let scope_bytes = snapshot_entry
            .scope
            .as_ref()
            .map_or(0, |scope| scope.path().total_segment_bytes());
        super::super::precompute::work::checkpoint(
            work.as_deref_mut(),
            source_scopes
                .saturating_add(1)
                .saturating_mul(scope_bytes.saturating_add(1)),
        )?;
        if !graph.is_alive(snapshot_entry.source) {
            meaningful_change_detected = true;
            continue;
        }

        if !matches!(graph.get_state(snapshot_entry.source)?, NodeState::Clean) {
            requires_upstream_evaluation.push(snapshot_entry.source);
            continue;
        }

        let current_version = graph.node_version_for_scope(
            snapshot_entry.source,
            snapshot_entry.aspect,
            snapshot_entry.scope.as_ref(),
        )?;
        if let Some(scope) = &snapshot_entry.scope {
            if current_version == snapshot_entry.cached_version {
                continue;
            }
            if partition_scope_untouched(
                graph.node_runtime_artifact_hot(snapshot_entry.source)?,
                scope,
            ) {
                continue;
            }
            meaningful_change_detected = true;
            continue;
        }

        if comparator.has_meaningful_change(
            snapshot_entry.aspect,
            snapshot_entry.cached_version,
            current_version,
            resolver,
        )? {
            meaningful_change_detected = true;
        }
    }

    super::super::precompute::work::checkpoint(
        work,
        requires_upstream_evaluation.len().saturating_mul(
            requires_upstream_evaluation
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize
                + 2,
        ),
    )?;
    requires_upstream_evaluation =
        DedupedNodeBatch::canonicalize_unordered(requires_upstream_evaluation).into_vec();

    Ok(MaybeStalePreview {
        unchanged: !meaningful_change_detected && requires_upstream_evaluation.is_empty(),
        requires_upstream_evaluation,
    })
}

pub(crate) fn partition_scope_untouched(
    trace_summary: Option<&crate::data::trace::RuntimeArtifactHot>,
    scope: &PartitionSubscription,
) -> bool {
    trace_summary.is_none_or(|summary| {
        !summary
            .changed_scopes
            .as_slice()
            .iter()
            .any(|changed_scope| crate::data::output::scopes_overlap(scope, changed_scope))
    })
}
