use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionResourceLease, MapKernelContext, MapKernelFailure,
    MapOutcome, MapPartition, PreparedExecutionMap,
};
use worth_foundational::PartitionIdentity;

use crate::clock::RuntimeInstant;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::{SignalError, SignalExecutionStop, SignalPublicationDisposition};
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::progression::DisjointGraphBatch;
use crate::logic::evaluation::{
    collect_effect_dependency_inputs_iter, proposed_effect_dependency_inputs,
};
use crate::logic::planner::precompute::graph_batch::CheckedApplyCapacity;
use crate::logic::planner::semantic::StageSemanticIdentity;
use crate::logic::planner::types::{
    ConcurrentApplyPlan, EligibleTask, ExecutionReport, LoweredTask, PlanSummary,
    ResolvedSignalPlannerPolicy, StageExecutionRecord,
};

use super::super::workspace::StageScratch;
use super::concurrent_packets;

pub(in crate::logic::planner) type PreparedSignalApplyMap<'authority> = PreparedExecutionMap<
    'authority,
    usize,
    crate::data::proof::invalidation::progression::GraphProposalKey,
    crate::logic::planner::apply::workspace::GroupLocalApplyPacket,
    SignalError,
>;

pub(in crate::logic::planner) fn prepare_checked_apply_map<'authority>(
    tasks: &[EligibleTask],
    batch: &DisjointGraphBatch,
    apply: &CheckedApplyCapacity,
    lease: &ExecutionResourceLease<'authority>,
    policy: &ResolvedSignalPlannerPolicy,
    request_work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<PreparedSignalApplyMap<'authority>, SignalError> {
    crate::logic::planner::precompute::work::checkpoint(
        request_work,
        tasks
            .len()
            .saturating_mul(crate::data::aspect::MAX_ASPECTS + 8),
    )?;
    if let Some(preparation) = preparation {
        preparation.claim_vec::<MapPartition<usize, crate::data::proof::invalidation::progression::GraphProposalKey>>(tasks.len())?;
        preparation.claim_vec::<PartitionIdentity>(tasks.len())?;
        for task in tasks {
            preparation
                .claim_vec::<crate::data::proof::invalidation::progression::GraphProposalKey>(
                    admitted_effect_keys(batch, task.node)?.len(),
                )?;
        }
    }
    let map = admitted_packet_map(tasks, batch, apply)?;
    let posture = if tasks.len() < policy.full_parallel_min_tasks() {
        worth_foundational::ExecutionPosture::Serial
    } else {
        lease.policy().posture()
    };
    let child = lease
        .child(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                posture,
                lease.policy().determinism(),
                lease.policy().budget(),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .map_err(SignalError::ExecutionAdmissionDenied)?;
    map.prepare_run(child)
        .map_err(SignalError::ExecutionAdmissionDenied)
}

pub(super) fn run_grouped_concurrent_apply_pass(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    stage_index: u32,
    tasks: Vec<LoweredTask>,
    plan: ConcurrentApplyPlan,
    lease: &ExecutionResourceLease<'_>,
    _policy: &ResolvedSignalPlannerPolicy,
    _batch: &DisjointGraphBatch,
    apply: &CheckedApplyCapacity,
    prepared_map: PreparedSignalApplyMap<'_>,
    candidates: crate::data::graph::PreparedCandidateEpoch<'_>,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    stage_identities: &[StageSemanticIdentity],
    report: &mut ExecutionReport,
    stage_record: &mut StageExecutionRecord,
    mut request_work: Option<&mut MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<StageScratch, SignalError> {
    let proposal_work = tasks
        .iter()
        .try_fold(tasks.len(), |total, task| {
            total.checked_add(task.dependency_inputs().as_slice().len())
        })
        .ok_or_else(|| SignalError::invalid_input("graph proposal work overflow"))?;
    crate::logic::planner::precompute::work::checkpoint(
        request_work.as_deref_mut(),
        proposal_work,
    )?;
    if let Some(budget) = preparation.as_deref_mut() {
        budget
            .claim_vec::<(NodeId, crate::data::dependency::CanonicalDependencies)>(tasks.len())?;
        budget.claim_vec::<(NodeId, &[crate::data::dependency::DependencyEdge])>(tasks.len())?;
        for task in &tasks {
            let edges = task.dependency_inputs().as_slice();
            budget.claim_vec::<crate::data::dependency::DependencyEdge>(edges.len())?;
            for edge in edges {
                if let Some(scope) = edge.scope_ref() {
                    let path = scope.path();
                    let bytes = path
                        .depth()
                        .checked_mul(std::mem::size_of::<String>())
                        .and_then(|base| base.checked_add(path.checked_segment_bytes()?))
                        .and_then(|bytes| u64::try_from(bytes).ok())
                        .ok_or_else(|| {
                            SignalError::invalid_input("proposal scope clone size overflow")
                        })?;
                    budget.claim(bytes)?;
                }
            }
        }
    }
    let proposals = tasks
        .iter()
        .map(|task| (task.node(), task.dependency_inputs().clone()))
        .collect::<Vec<_>>();
    let proposal_refs = proposals
        .iter()
        .map(|(node, desired)| (*node, desired.as_slice()))
        .collect::<Vec<_>>();
    let topology = graph.prepare_dependency_topology_epoch(
        &proposal_refs,
        request_work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    stage_record.apply_mode =
        Some(crate::logic::planner::ParallelApplyMode::GroupedConcurrentApply);
    stage_record.apply_group_count = plan.groups.len() as u32;
    stage_record.concurrent_apply_task_count = tasks.len() as u32;

    let dependency_input_start = RuntimeInstant::now();
    let dependency_inputs = if tasks
        .iter()
        .any(|task| task.execution().dependency_updates() != 0)
    {
        proposals
            .iter()
            .map(|(node, desired)| {
                proposed_effect_dependency_inputs(graph, *node, desired.as_slice())
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        collect_effect_dependency_inputs_iter(graph, tasks.iter().map(LoweredTask::node))?
    };
    let dependency_input_nanos = dependency_input_start.elapsed().as_nanos();
    graph.with_telemetry(|telemetry| {
        telemetry.execution.dependency_input_build_nanos += dependency_input_nanos
    });
    let group_inputs = concurrent_packets::build_concurrent_apply_group_inputs(
        tasks,
        dependency_inputs,
        &plan.groups,
        stage_identities,
    )?;
    if group_inputs.len() != prepared_map.partition_count() {
        return Err(SignalError::internal(
            "apply groups differ from prepared map",
        ));
    }
    for group in &group_inputs {
        let scratch = group
            .task_indices()
            .try_fold(0_u64, |total, index| {
                total.checked_add(apply.members().get(index)?.scratch_bytes())
            })
            .ok_or_else(|| {
                SignalError::internal("apply group escaped prepared scratch capacity")
            })?;
        if group.additional_charged_bytes() > scratch {
            return Err(SignalError::invalid_input(
                "parallel apply input exceeded admitted scratch capacity",
            ));
        }
    }
    let graph_ref = &*graph;
    let outcome = prepared_map.run(|index, work| {
        work.checkpoint(1).map_err(MapKernelFailure::Stop)?;
        let clone_bytes = group_inputs[*index].additional_charged_bytes();
        work.checkpoint(clone_bytes)
            .map_err(MapKernelFailure::Stop)?;
        concurrent_packets::build_group_packet(graph_ref, group_inputs[*index].clone(), work)
            .map_err(|failure| MapKernelFailure::Domain(failure.error))
    });
    let (packets, execution) = match outcome {
        MapOutcome::Complete { values, report } => (values, report),
        MapOutcome::Stopped {
            boundary,
            reason,
            report,
            ..
        } => {
            return Err(SignalError::execution_stopped(SignalExecutionStop::new(
                reason.into(),
                boundary,
                SignalPublicationDisposition::WorkerLocal,
                report,
            )));
        }
    };
    let apply_parallel =
        execution.resolved_posture() == worth_foundational::ExecutionPosture::Automatic;
    if apply_parallel {
        stage_record.outcome = crate::logic::planner::StageExecutionOutcome::CompletedParallel;
        stage_record.parallel_kind =
            Some(crate::logic::planner::ParallelExecutionKind::FullParallel);
        graph.with_telemetry(|telemetry| telemetry.execution.parallel_stage_dispatch_count += 1);
    } else if matches!(
        stage_record.outcome,
        crate::logic::planner::StageExecutionOutcome::CompletedParallel
    ) {
        stage_record.parallel_kind =
            Some(crate::logic::planner::ParallelExecutionKind::StagedParallelPrecompute);
    }
    stage_record.parallel_admission_reason =
        Some(crate::logic::planner::ParallelAdmissionReason::AdmittedProofSafeGroupedConcurrent);
    let packet_count = packets.len() as u64;
    graph.with_telemetry(|telemetry| {
        telemetry.execution.group_local_packet_breadth += packets
            .iter()
            .map(|packet| packet.packet_breadth() as u64)
            .sum::<u64>();
        telemetry.execution.reduction_packet_breadth += packet_count;
        telemetry.execution.reduction_group_count += packet_count;
    });
    report.execution.push(execution);
    concurrent_packets::reduce_grouped_concurrent_packets(
        graph,
        summary,
        stage_index,
        topology,
        packets,
        plan.reduction,
        comparator_resolver,
        Some(lease),
        candidates,
        request_work,
        preparation,
    )
}

fn admitted_packet_map(
    tasks: &[EligibleTask],
    batch: &DisjointGraphBatch,
    apply: &CheckedApplyCapacity,
) -> Result<
    ExecutionMap<usize, crate::data::proof::invalidation::progression::GraphProposalKey>,
    SignalError,
> {
    let mut partitions = Vec::with_capacity(tasks.len());
    for (index, task) in tasks.iter().enumerate() {
        let member = apply
            .members()
            .get(index)
            .ok_or_else(|| SignalError::internal("apply task escaped admitted epoch capacity"))?;
        let effects = admitted_effect_keys(batch, task.node)?;
        let mut write_keys = Vec::with_capacity(effects.len());
        write_keys.extend_from_slice(effects);
        write_keys.sort_unstable();
        write_keys.dedup();
        partitions.push(MapPartition {
            identity: PartitionIdentity::new(index as u64),
            value: index,
            read_keys: Vec::new(),
            write_keys,
            kernel_scratch_bytes: member.scratch_bytes(),
            max_result_bytes: member.result_bytes(),
        });
    }
    let identities = partitions
        .iter()
        .map(|partition| partition.identity)
        .collect();
    ExecutionMap::try_from_declared_partitions(identities, partitions).map_err(|_| {
        SignalError::invalid_input("parallel apply effect keys or memory admission denied")
    })
}

fn admitted_effect_keys(
    batch: &DisjointGraphBatch,
    node: NodeId,
) -> Result<&[crate::data::proof::invalidation::progression::GraphProposalKey], SignalError> {
    batch
        .effects()
        .binary_search_by_key(&node, |(target, _)| *target)
        .ok()
        .and_then(|position| batch.effects().get(position))
        .map(|(_, keys)| keys.as_slice())
        .ok_or_else(|| SignalError::internal("admitted graph batch omitted apply target"))
}
