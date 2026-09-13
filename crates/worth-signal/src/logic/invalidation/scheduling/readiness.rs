use worth_proof::TransitionOutcome;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::progression::{
    InvalidationOriginBinding, InvalidationProgressionDenial, InvalidationProgressionOwner,
    InvalidationReadinessEpoch, InvalidationStageOrder, LoweredInvalidationBatch,
    ReadyInvalidationBatch,
};
use crate::data::proof::invalidation::revalidation::NodeInvalidationInput;

pub(crate) fn admit_current_readiness(
    graph: &SignalGraph,
    lowered: LoweredInvalidationBatch,
    readiness_epoch: InvalidationReadinessEpoch,
    stage_order: InvalidationStageOrder,
) -> Result<ReadyInvalidationBatch, SignalError> {
    if readiness_epoch != graph.current_invalidation_readiness_epoch() {
        return Err(SignalError::invalid_input(
            "lowered invalidation belongs to a stale readiness epoch",
        ));
    }
    let expected = InvalidationProgressionOwner::binding_axes(&lowered);
    if matches!(
        graph.node_invalidation_input(expected.target)?,
        NodeInvalidationInput::Pending(_)
    ) {
        return Err(SignalError::invalid_input(
            "dependency invalidation is still pending",
        ));
    }
    let mut current = expected.clone();
    current.graph_instance = graph.runtime_instance_id();
    current.dependency_revision = graph.dependency_revision(expected.target)?;
    current.readiness_epoch = readiness_epoch;
    current.stage_order = stage_order;
    current.origin = current_origin(graph, expected.target, &expected.origin)?;
    match InvalidationProgressionOwner::admit_ready(lowered, current) {
        TransitionOutcome::Success(ready) => Ok(ready),
        TransitionOutcome::Denied(denial)
        | TransitionOutcome::Deferred(denial)
        | TransitionOutcome::Stale(denial)
        | TransitionOutcome::RebindRequired(denial) => Err(readiness_denial(denial)),
        TransitionOutcome::Failed(error) => Err(error),
    }
}

pub(super) fn ensure_ready_is_current(
    graph: &SignalGraph,
    ready: &ReadyInvalidationBatch,
) -> Result<(), SignalError> {
    let expected = InvalidationProgressionOwner::ready_binding(ready);
    if expected.graph_instance != graph.runtime_instance_id() {
        return Err(SignalError::invalid_input(
            "ready invalidation belongs to a stale graph instance",
        ));
    }
    if expected.dependency_revision != graph.dependency_revision(expected.target)? {
        return Err(SignalError::invalid_input(
            "ready invalidation belongs to a stale dependency revision",
        ));
    }
    if expected.readiness_epoch != graph.current_invalidation_readiness_epoch() {
        return Err(SignalError::invalid_input(
            "ready invalidation belongs to a stale readiness epoch",
        ));
    }
    if expected.origin != current_origin(graph, expected.target, &expected.origin)? {
        return Err(SignalError::invalid_input(
            "ready invalidation belongs to stale causal authority",
        ));
    }
    Ok(())
}

fn current_origin(
    graph: &SignalGraph,
    target: crate::data::handle::NodeId,
    expected: &InvalidationOriginBinding,
) -> Result<InvalidationOriginBinding, SignalError> {
    match expected {
        InvalidationOriginBinding::SourceAdmission { .. } => {
            let generation = graph
                .node_direct_invalidation_basis(target)?
                .map(|basis| basis.generation())
                .ok_or_else(|| {
                    SignalError::invalid_input("source invalidation basis was released")
                })?;
            Ok(InvalidationOriginBinding::SourceAdmission { generation })
        }
        InvalidationOriginBinding::DependencyCommit { .. } => {
            let mut producer_commit_ordinals = graph
                .pending_causes(target)?
                .iter()
                .map(|cause| cause.binding_axes.output_commit_ordinal)
                .collect::<Vec<_>>();
            producer_commit_ordinals.sort_unstable();
            producer_commit_ordinals.dedup();
            Ok(InvalidationOriginBinding::DependencyCommit {
                cause_set: graph.pending_cause_set_id(target)?,
                producer_commit_ordinals,
            })
        }
        InvalidationOriginBinding::StructuralMutation { .. } => {
            let pending = graph
                .pending_dependency_revalidation(target)?
                .ok_or_else(|| {
                    SignalError::invalid_input("structural invalidation was released")
                })?;
            if !pending.requires_structural_recompute() {
                return Err(SignalError::invalid_input(
                    "structural invalidation no longer requires recompute",
                ));
            }
            Ok(InvalidationOriginBinding::StructuralMutation {
                ordinal: pending.dependency_revision().0,
            })
        }
    }
}

fn readiness_denial(denial: InvalidationProgressionDenial) -> SignalError {
    SignalError::invalid_input(format!(
        "lowered invalidation did not retain current readiness: {denial:?}"
    ))
}
