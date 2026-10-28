//! Publication tickets held across checked callbacks under a finite grant.
use crate::data::error::SignalError;
use crate::data::graph::{PreparedCandidateEpoch, SignalGraph};
use crate::data::proof::invalidation::progression::DisjointGraphBatch;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::logic::planner::apply::stage::{prepare_checked_apply_map, PreparedSignalApplyMap};
use crate::logic::planner::execution::{CheckedAdmittedEpoch, EpochMetadata};
use crate::logic::planner::precompute::graph_batch::CheckedApplyCapacity;
use crate::logic::planner::types::ResolvedSignalPlannerPolicy;
use worth_execution::{ExecutionRequest, MapKernelContext};

pub(super) struct PreparedPublication<'tasks, 'request, 'authority> {
    pub metadata: EpochMetadata<'tasks>,
    pub request: ExecutionRequest<'request, 'authority>,
    pub apply: CheckedApplyCapacity,
    pub prepared_map: PreparedSignalApplyMap<'request, 'authority>,
    pub candidates: PreparedCandidateEpoch<'request, 'authority>,
}

pub(super) enum PublicationCapacity<'tasks, 'request, 'authority> {
    Reserved(PreparedPublication<'tasks, 'request, 'authority>),
    Unbounded(CheckedAdmittedEpoch<'tasks, 'request, 'authority>),
}

impl<'tasks, 'request, 'authority> PublicationCapacity<'tasks, 'request, 'authority> {
    pub(super) fn admit(
        graph: &mut SignalGraph,
        checked: CheckedAdmittedEpoch<'tasks, 'request, 'authority>,
        batch: &DisjointGraphBatch,
        policy: &ResolvedSignalPlannerPolicy,
        work: Option<&mut MapKernelContext<'_, '_>>,
        preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<Self, SignalError> {
        match checked.result_grant_bytes() {
            Some(bound) => {
                prepare(graph, checked, batch, bound, policy, work, preparation).map(Self::Reserved)
            }
            None => Ok(Self::Unbounded(checked)),
        }
    }

    pub(super) fn settle(
        self,
        graph: &mut SignalGraph,
        batch: &DisjointGraphBatch,
        measured_result_bytes: u64,
        policy: &ResolvedSignalPlannerPolicy,
        work: Option<&mut MapKernelContext<'_, '_>>,
        preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<PreparedPublication<'tasks, 'request, 'authority>, SignalError> {
        match self {
            Self::Reserved(publication) => Ok(publication),
            Self::Unbounded(checked) => prepare(
                graph,
                checked,
                batch,
                measured_result_bytes,
                policy,
                work,
                preparation,
            ),
        }
    }
}

pub(super) fn prepare<'tasks, 'request, 'authority>(
    graph: &mut SignalGraph,
    checked: CheckedAdmittedEpoch<'tasks, 'request, 'authority>,
    batch: &DisjointGraphBatch,
    result_bytes: u64,
    policy: &ResolvedSignalPlannerPolicy,
    mut work: Option<&mut MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<PreparedPublication<'tasks, 'request, 'authority>, SignalError> {
    let (metadata, request, apply) = checked.into_prepared_parts(result_bytes)?;
    let prepared_map = prepare_checked_apply_map(
        metadata.tasks(),
        batch,
        &apply,
        request,
        policy,
        work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    let candidates = graph.prepare_candidate_epoch(
        metadata.tasks().iter().map(|task| task.node),
        result_bytes,
        if metadata.tasks().len() < policy.full_parallel_min_tasks() {
            worth_foundational::ExecutionPosture::Serial
        } else {
            request.resolved_posture()
        },
        request,
        work.expect("checked request work"),
        preparation.expect("checked preparation owner"),
    )?;
    Ok(PreparedPublication {
        metadata,
        request,
        apply,
        prepared_map,
        candidates,
    })
}
