use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::MemoizedResultOrigin;
use crate::data::proof::SnapshotBatchCommit;
use crate::data::reuse::ReuseBasis;
use crate::logic::evaluation::{
    build_prepared_apply_commit_packet, record_reuse_rejection_telemetry, ApplyCommitBuildError,
    EffectDependencyInputs,
};
use crate::logic::planner::semantic::StageSemanticIdentity;
use crate::logic::planner::semantic::{
    segment_for_single_update, SemanticTaskUpdate, StageSemanticBatch,
};
use crate::logic::planner::types::{
    ConcurrentApplyReductionPlan, DisjointApplyGroup, LoweredTask, PlanSummary,
    ReductionOrderingContract, ReductionWorkClass,
};
use worth_execution::{ExecutionResourceLease, MapKernelContext};

use crate::logic::planner::apply::workspace::{
    ConcurrentApplyGroupInput, ConcurrentWorkerInput, GroupLocalApplyPacket, GroupLocalTaskCommit,
    GroupedApplyFailure, StageFinalizeWork, StageScratch,
};

pub(super) use crate::logic::planner::apply::groups::build_stage_apply_groups;

pub(super) fn build_group_packet(
    graph: &SignalGraph,
    group: ConcurrentApplyGroupInput,
    work: &mut MapKernelContext<'_, '_>,
) -> Result<GroupLocalApplyPacket, GroupedApplyFailure> {
    let (group_index, worker_inputs) = group.into_parts();
    let mut task_commits = Vec::with_capacity(worker_inputs.len());
    let mut checkpoint = |visits: usize| {
        let units = u64::try_from(visits)
            .map_err(|_| SignalError::invalid_input("grouped apply work bound overflow"))?;
        work.checkpoint(units)
            .map_err(|_| SignalError::invalid_input("grouped apply request work stopped"))
    };
    let mut evaluation_work =
        crate::logic::evaluation::EvaluationWork::RequestCheckpoint(&mut checkpoint);
    for worker_input in worker_inputs {
        let (
            task_index,
            node,
            identity,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
            comparator_policy,
            prepared,
            dependency_inputs,
        ) = worker_input.into_parts();
        let commit_packet = build_prepared_apply_commit_packet(
            graph,
            node,
            prepared,
            comparator_policy,
            None,
            dependency_updates,
            dependency_inputs,
            false,
            &mut evaluation_work,
        )
        .map_err(|error| grouped_apply_failure_from_build_error(node, identity.record_id, error))?;
        task_commits.push(GroupLocalTaskCommit::new(
            task_index,
            node,
            identity,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
            commit_packet.into(),
        ));
    }
    Ok(GroupLocalApplyPacket::new(group_index, task_commits))
}

pub(super) fn reduce_grouped_concurrent_packets(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    stage_index: u32,
    topology: crate::data::graph::PreparedDependencyTopologyEpoch,
    mut packets: Vec<GroupLocalApplyPacket>,
    reduction: ConcurrentApplyReductionPlan,
    comparator_resolver: &mut impl crate::data::comparator::ComparatorPolicyResolver,
    lease: Option<&ExecutionResourceLease<'_>>,
    candidates: crate::data::graph::PreparedCandidateEpoch<'_>,
    mut request_work: Option<&mut MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<StageScratch, SignalError> {
    debug_assert!(
        matches!(
            reduction.allowed_work,
            ReductionWorkClass::DeterministicPublicationOnly
        ),
        "grouped concurrent reduction may only perform deterministic publication"
    );
    match reduction.ordering_contract {
        ReductionOrderingContract::StageTaskIndexOrder => {
            packets.sort_by_key(|packet| packet.group_index());
        }
    }

    let mut commits = packets
        .into_iter()
        .flat_map(GroupLocalApplyPacket::into_task_commits)
        .collect::<Vec<_>>();
    commits.sort_by_key(GroupLocalTaskCommit::task_index);
    let mut metadata = Vec::with_capacity(commits.len());
    let mut epoch_packets = Vec::with_capacity(commits.len());
    for commit in commits {
        let (
            task_index,
            node,
            identity,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
            packet,
        ) = commit.into_parts();
        metadata.push((
            task_index,
            node,
            identity,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
        ));
        epoch_packets.push(packet);
    }
    if let Some(budget) = preparation.as_deref_mut() {
        use crate::data::retained_storage::{
            RetainedStorageMeasurement, RetainedStoragePreparation,
            RetainedStoragePreparationDenial,
        };
        budget.claim_vec::<crate::data::graph::EpochSemanticSeed>(metadata.len())?;
        let mut measurement = RetainedStoragePreparation::new(usize::MAX);
        let mut checkpoint = |visits: usize| {
            crate::logic::planner::precompute::work::checkpoint(request_work.as_deref_mut(), visits)
                .map_err(|_| RetainedStoragePreparationDenial::WorkExhausted {
                    maximum_visits: usize::MAX,
                })
        };
        let mut observed = measurement.reborrow_with_checkpoint(&mut checkpoint);
        for (_, _, _, _, before_image, _, _, _, rewiring) in &metadata {
            let bytes = before_image
                .retained_heap_charge(&mut observed)
                .and_then(|charge| {
                    charge.checked_add(rewiring.retained_heap_charge(&mut observed)?)
                })
                .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            budget.claim(bytes.bytes())?;
        }
    }
    let semantic_seeds = metadata
        .iter()
        .map(|(_, node, identity, _, before_image, _, _, _, rewiring)| {
            crate::data::graph::EpochSemanticSeed {
                node: *node,
                execution_record: identity.record_id,
                semantic_segment: identity.segment_id,
                before_image: before_image.clone(),
                rewiring: rewiring.clone(),
            }
        })
        .collect();
    let epoch = graph
        .prepare_parallel_apply_epoch(
            topology,
            epoch_packets,
            semantic_seeds,
            comparator_resolver,
            lease,
            candidates,
            request_work,
            preparation,
        )
        .inspect_err(|error| {
            if let Some((_, node, identity, ..)) = metadata.first() {
                record_grouped_apply_failure(
                    graph,
                    summary,
                    stage_index,
                    &GroupedApplyFailure {
                        node: *node,
                        record_id: identity.record_id,
                        reuse_failure: None,
                        error: SignalError::internal(error.to_string()),
                    },
                );
            }
        })?;
    let outcomes = epoch.publish(graph);
    assert_eq!(metadata.len(), outcomes.len(), "one epoch outcome per task");
    let mut semantic_batch = StageSemanticBatch::default();
    let mut pending_snapshots = Vec::new();
    for (
        (
            task_index,
            node,
            identity,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
        ),
        (report, pending_snapshot, prepared_artifacts),
    ) in metadata.into_iter().zip(outcomes)
    {
        if let Some(snapshot) = pending_snapshot {
            pending_snapshots.push(snapshot);
        }
        let after_state = graph.get_state(node).expect("prevalidated epoch task node");
        let after_trace = graph
            .node_runtime_artifact_operational_summary(node)
            .expect("prevalidated epoch artifact state");
        let memoized_origin = after_trace
            .as_ref()
            .map(|trace| trace.memoized_origin)
            .unwrap_or(MemoizedResultOrigin::DirectCompute);
        let reuse_basis = after_trace
            .map(|trace| trace.reuse_basis)
            .unwrap_or_else(ReuseBasis::fresh_compute);
        semantic_batch.push_segment(segment_for_single_update(
            SemanticTaskUpdate::new(
                task_index,
                node,
                identity,
                before_state,
                before_artifact_state,
                after_state,
                dependency_updates,
                recomputed,
                partition_aware,
                report.temporal_eligibility,
                rewiring,
                report.verdict,
                memoized_origin,
                reuse_basis,
            )
            .with_prepared_artifacts(prepared_artifacts),
        ));
    }
    let publication_breadth =
        semantic_batch.segment_count() as u64 + pending_snapshots.len() as u64;
    graph.with_telemetry(|telemetry| {
        telemetry.execution.shared_surface_publication_breadth += publication_breadth;
    });
    Ok(StageScratch::new(
        StageFinalizeWork::Parallel(crate::data::proof::SingleConsumer::new(semantic_batch)),
        SnapshotBatchCommit::from_unique_pending_snapshots_in_stage_order(pending_snapshots)
            .classify(),
    ))
}

fn grouped_apply_failure_from_build_error(
    node: NodeId,
    record_id: crate::logic::planner::ExecutionRecordId,
    error: ApplyCommitBuildError,
) -> GroupedApplyFailure {
    GroupedApplyFailure {
        node,
        record_id,
        reuse_failure: error.reuse_failure(),
        error: error.into_signal(),
    }
}

pub(super) fn record_grouped_apply_failure(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    stage_index: u32,
    failure: &GroupedApplyFailure,
) {
    if let Some(reuse_failure) = failure.reuse_failure {
        record_reuse_rejection_telemetry(graph, &reuse_failure);
    }
    crate::logic::planner::execution::task_reporting::record_execution_failure_if_enabled(
        graph,
        || {
            crate::diagnostics::failure::ExecutionFailureContext::new(
                crate::diagnostics::failure::ExecutionFailurePhase::Apply,
                Some(stage_index),
                Some(failure.node),
                None,
                Some(failure.record_id),
                Some(*summary),
                failure.error.to_string(),
            )
        },
    );
}

pub(super) fn build_concurrent_apply_group_inputs(
    tasks: Vec<LoweredTask>,
    dependency_inputs: Vec<EffectDependencyInputs>,
    groups: &[DisjointApplyGroup],
    stage_identities: &[StageSemanticIdentity],
) -> Result<Vec<ConcurrentApplyGroupInput>, SignalError> {
    let mut task_slots = tasks.into_iter().map(Some).collect::<Vec<_>>();
    let mut dependency_input_slots = dependency_inputs.into_iter().map(Some).collect::<Vec<_>>();
    let mut group_inputs = Vec::with_capacity(groups.len());
    for (group_index, group) in groups.iter().enumerate() {
        let mut worker_inputs = Vec::with_capacity(group.task_indices.len());
        for &task_index in &group.task_indices {
            let lowered_task = take_slot(
                &mut task_slots[task_index],
                "grouped concurrent lowered task slot was consumed more than once",
            )?;
            let dependency_input = take_slot(
                &mut dependency_input_slots[task_index],
                "grouped concurrent dependency inputs no longer align with lowered tasks",
            )?;
            worker_inputs.push(
                lowered_task
                    .into_concurrent_worker_input(stage_identities[task_index], dependency_input),
            );
        }
        group_inputs.push(ConcurrentApplyGroupInput::new(group_index, worker_inputs));
    }
    Ok(group_inputs)
}

fn take_slot<T>(slot: &mut Option<T>, context: &'static str) -> Result<T, SignalError> {
    slot.take().ok_or_else(|| SignalError::internal(context))
}

impl LoweredTask {
    fn into_concurrent_worker_input(
        self,
        identity: StageSemanticIdentity,
        dependency_inputs: EffectDependencyInputs,
    ) -> ConcurrentWorkerInput {
        let (
            task_index,
            node,
            _produced_aspects,
            _dependency_inputs,
            comparator_policy,
            _path_class,
            _authority_policy,
            _footprint,
            execution,
        ) = self.into_parts();
        let (
            prepared,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
        ) = execution.into_parts();
        ConcurrentWorkerInput::new(
            task_index,
            node,
            identity,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
            comparator_policy,
            prepared,
            dependency_inputs,
        )
    }
}
