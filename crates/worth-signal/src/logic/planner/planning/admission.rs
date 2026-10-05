use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::{ContextRequirement, NodeState};
use crate::logic::evaluation::EvaluationRequestMode;

use super::super::types::{EligibleTask, EligibleTaskAdmission, MaybeStaleAdmission, TaskReason};
use super::validation::preview_maybe_stale;

pub(super) fn verify_required_context(
    node: NodeId,
    requirement: ContextRequirement,
) -> Result<(), SignalError> {
    match requirement {
        ContextRequirement::None | ContextRequirement::DomainContext => Ok(()),
        ContextRequirement::RelationalSnapshot => {
            Err(SignalError::contract_violation(node, requirement))
        }
    }
}

pub(super) fn admit_planned_node(
    graph: &SignalGraph,
    node: NodeId,
    direct_request: bool,
    request_mode: EvaluationRequestMode,
    maybe_stale_admission: Option<MaybeStaleAdmission>,
) -> Result<EligibleTask, SignalError> {
    let dirty_partition_scopes_present = graph.node_dirty_partition_scopes_present(node)?;
    let node_state_at_admission = Some(graph.get_state(node)?);
    let reason = classify_reason(graph, node, direct_request, request_mode)?;
    Ok(EligibleTask {
        node,
        request_mode,
        direct_request,
        reason,
        admission: EligibleTaskAdmission {
            node_state_at_admission,
            dirty_partition_scopes_present,
            maybe_stale: maybe_stale_admission,
        },
    })
}

pub(crate) fn admit_direct_task_with_policy_resolver(
    graph: &SignalGraph,
    node: NodeId,
    request_mode: EvaluationRequestMode,
    resolver: &mut impl ComparatorPolicyResolver,
) -> Result<EligibleTask, SignalError> {
    let state = graph.get_state(node)?;
    let maybe_stale_admission = if matches!(state, NodeState::MaybeStale) {
        let preview = preview_maybe_stale(graph, node, resolver, None, None)?;
        Some(MaybeStaleAdmission {
            unchanged_at_admission: preview.unchanged,
        })
    } else {
        None
    };
    admit_planned_node(graph, node, true, request_mode, maybe_stale_admission)
}

fn classify_reason(
    graph: &SignalGraph,
    node: NodeId,
    direct_request: bool,
    request_mode: EvaluationRequestMode,
) -> Result<TaskReason, SignalError> {
    if direct_request {
        return Ok(match request_mode {
            EvaluationRequestMode::Default => TaskReason::RequestedTarget,
            EvaluationRequestMode::ForceOnDemand => TaskReason::ConditionForced,
        });
    }

    let state = graph.get_state(node)?;
    if matches!(state, NodeState::MaybeStale) {
        return Ok(TaskReason::MaybeStaleValidation);
    }
    if matches!(state, NodeState::Clean) {
        return Ok(TaskReason::DependencyRequired);
    }

    if graph.node_dirty_partition_scopes_present(node)? {
        return Ok(TaskReason::PartitionScopedDependency);
    }

    let hot_trace = graph.node_runtime_artifact_hot(node)?;
    if hot_trace.is_some_and(|summary| {
        summary.output_change == crate::data::output::OutputChange::Unchanged
    }) {
        return Ok(TaskReason::OutputDiffDependent);
    }

    let operational_summary = graph.node_runtime_artifact_operational_summary(node)?;
    if operational_summary.is_some_and(|summary| {
        summary.reuse_basis.source == crate::data::reuse::ReuseSource::MemoizedArtifact
    }) {
        return Ok(TaskReason::MemoValidation);
    }

    Ok(TaskReason::Dirty)
}
