use super::callback::SignalPrecompute;
use super::eligibility::{prevalidate_stage_tasks, PrevalidatedTask};
use super::TemporalLoweringContext;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::{SignalError, SignalExecutionStop, SignalPublicationDisposition};
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::progression::{DisjointGraphBatch, GraphProposalKey};
use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};
use crate::logic::planner::execution::{AdmittedEpoch, EpochMetadata};
use crate::logic::planner::types::{EligibleTask, ResolvedSignalPlannerPolicy};
use crate::logic::prepared::{ExecutionSnapshot, PreparedEvaluation};
use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionResourceLease, MapKernelContext, MapKernelFailure,
    MapOutcome, MapPartition,
};
use worth_foundational::PartitionIdentity;

mod capacity;
pub(super) use capacity::{checked_map_memory_requirement, PrecomputeMapBasis};

pub(super) enum PreparedEpoch<'tasks, 'lease, 'authority> {
    Checked {
        metadata: EpochMetadata<'tasks>,
        values: Vec<PreparedEvaluation>,
        batch: DisjointGraphBatch,
        lease: &'lease ExecutionResourceLease<'authority>,
        apply: super::graph_batch::CheckedApplyCapacity,
        prepared_map: crate::logic::planner::apply::stage::PreparedSignalApplyMap<'authority>,
        candidates: crate::data::graph::PreparedCandidateEpoch<'authority>,
        reports: Vec<worth_foundational::ExecutionReport>,
    },
    LegacySerial {
        metadata: EpochMetadata<'tasks>,
        values: Vec<PreparedEvaluation>,
    },
}

struct GraphWorkItem {
    task_index: usize,
    capacity: super::callback::CheckedKernelCapacity,
}
impl ChargedBytes for GraphWorkItem {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

pub(super) fn prepare_epoch<'tasks, 'lease, 'authority>(
    graph: &mut SignalGraph,
    stage: AdmittedEpoch<'tasks, 'lease, 'authority>,
    precompute: &impl SignalPrecompute,
    comparator: &mut impl ComparatorPolicyResolver,
    temporal: &TemporalLoweringContext,
    policy: &ResolvedSignalPlannerPolicy,
    mut request_work: Option<&mut MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<PreparedEpoch<'tasks, 'lease, 'authority>, SignalError> {
    preparation_budget::claim_vec::<PrevalidatedTask>(
        preparation.as_deref_mut(),
        stage.tasks().len(),
    )?;
    let epoch = stage.readiness_epoch();
    let validated = prevalidate_stage_tasks(
        graph,
        stage.tasks(),
        stage.index(),
        stage.task_offset(),
        epoch,
        comparator,
        temporal,
        request_work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    let checked = match stage {
        AdmittedEpoch::Checked(checked) => checked,
        AdmittedEpoch::LegacySerial(legacy) => {
            return prepare_legacy_epoch(graph, legacy, validated, precompute);
        }
    };
    let lease = checked.lease();
    let batch = super::graph_batch::admit(
        graph,
        checked.tasks(),
        checked.task_offset(),
        &validated,
        request_work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    let prepared_map = crate::logic::planner::apply::stage::prepare_checked_apply_map(
        checked.tasks(),
        &batch,
        checked.apply_capacity(),
        checked.lease(),
        policy,
        request_work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    let candidates = graph.prepare_candidate_epoch(
        checked.tasks().iter().map(|task| task.node),
        checked.result_grant_bytes(),
        if checked.tasks().len() < policy.full_parallel_min_tasks() {
            worth_foundational::ExecutionPosture::Serial
        } else {
            lease.policy().posture()
        },
        lease,
        request_work
            .as_deref_mut()
            .expect("checked request has work"),
        preparation
            .as_deref_mut()
            .expect("checked request has preparation"),
    )?;
    let map = lower_checked_map(
        graph,
        &validated,
        &batch,
        &checked,
        request_work.as_deref_mut(),
        preparation.as_deref_mut(),
    )?;
    let snapshot = ExecutionSnapshot::new(graph);
    let serial_lease = if map.partition_count() < policy.parallel_min_tasks() {
        Some(
            lease
                .child(worth_execution::LeaseRequest {
                    policy: worth_foundational::ExecutionRequestPolicy::new(
                        worth_foundational::ExecutionPosture::Serial,
                        lease.policy().determinism(),
                        lease.policy().budget(),
                    ),
                    deadline: None,
                    cancellation: worth_execution::CancellationToken::new(),
                })
                .map_err(|_| SignalError::invalid_input("graph serial child lease denied"))?,
        )
    } else {
        None
    };
    let outcome = map.run(serial_lease.as_ref().or(Some(lease)), |item, work| {
        let task = &checked.tasks()[item.task_index];
        work.checkpoint(1).map_err(MapKernelFailure::Stop)?;
        let view = snapshot.read_view(task.node);
        precompute
            .prepare(task.node, &view, Some(work), Some(item.capacity))
            .map_err(MapKernelFailure::Domain)
    });
    let (computed, execution) = match outcome {
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
    let values = reconcile_prepared_values(
        graph,
        checked.tasks(),
        validated,
        computed,
        request_work,
        preparation.as_deref_mut(),
    )?;
    preparation_budget::claim_vec::<worth_foundational::ExecutionReport>(preparation, 1)?;
    let (metadata, lease, apply) = checked.into_prepared_parts();
    Ok(PreparedEpoch::Checked {
        metadata,
        values,
        batch,
        lease,
        apply,
        prepared_map,
        candidates,
        reports: vec![execution],
    })
}

fn prepare_legacy_epoch<'tasks>(
    graph: &SignalGraph,
    admitted: crate::logic::planner::execution::LegacySerialEpoch<'tasks>,
    validated: Vec<PrevalidatedTask>,
    precompute: &impl SignalPrecompute,
) -> Result<PreparedEpoch<'tasks, 'static, 'static>, SignalError> {
    let tasks = admitted.tasks();
    let snapshot = ExecutionSnapshot::new(graph);
    let mut values = Vec::with_capacity(tasks.len());
    for (task, validation) in tasks.iter().zip(validated) {
        match validation {
            PrevalidatedTask::Prepared(prepared) => values.push(prepared),
            PrevalidatedTask::NeedsCompute {
                temporal_ready,
                ready_invalidation,
            } => {
                let view = snapshot.read_view(task.node);
                let compute = || precompute.prepare(task.node, &view, None, None);
                let mut prepared = match ready_invalidation {
                    Some(ready) => crate::logic::invalidation::scheduling::execute_ready(
                        graph, ready, compute,
                    )?,
                    None => compute()?,
                };
                if let Some(ready) = temporal_ready {
                    prepared = prepared.with_temporal_eligibility(
                        crate::data::temporal::LoweredTemporalEligibility::Ready(ready),
                    );
                }
                values.push(prepared);
            }
        }
    }
    Ok(PreparedEpoch::LegacySerial {
        metadata: admitted.into_metadata(),
        values,
    })
}

fn lower_checked_map(
    graph: &SignalGraph,
    validated: &[PrevalidatedTask],
    batch: &DisjointGraphBatch,
    stage: &crate::logic::planner::execution::CheckedAdmittedEpoch<'_, '_, '_>,
    mut work: Option<&mut MapKernelContext<'_, '_>>,
    mut preparation: Option<&mut SignalPreparationBudget>,
) -> Result<ExecutionMap<GraphWorkItem, GraphProposalKey>, SignalError> {
    let count = validated
        .iter()
        .filter(|task| matches!(task, PrevalidatedTask::NeedsCompute { .. }))
        .count();
    super::work::checkpoint(work.as_deref_mut(), validated.len().saturating_add(count))?;
    preparation_budget::claim_vec::<MapPartition<GraphWorkItem, GraphProposalKey>>(
        preparation.as_deref_mut(),
        count,
    )?;
    preparation_budget::claim_vec::<PartitionIdentity>(preparation.as_deref_mut(), count)?;
    let mut partitions = Vec::with_capacity(count);
    for (index, (task, validation)) in stage.tasks().iter().zip(validated).enumerate() {
        if !matches!(validation, PrevalidatedTask::NeedsCompute { .. }) {
            continue;
        }
        let declaration = graph
            .get_contract(task.node)?
            .execution
            .bounded_inputs
            .as_ref()
            .expect("admitted");
        let capture_bytes =
            crate::logic::prepared::PreparedDependencyCapture::checked_capture_heap_bound(
                declaration,
            )
            .ok_or_else(|| SignalError::invalid_input("checked capture memory overflow"))?;
        let per_result = stage
            .result_grant_bytes()
            .checked_add(capture_bytes)
            .ok_or_else(|| SignalError::invalid_input("checked result grant overflow"))?;
        let result_bytes = per_result.checked_sub(capture_bytes).ok_or_else(|| {
            SignalError::invalid_input("checked capture exceeds admitted result capacity")
        })?;
        let scratch_bytes = per_result
            .checked_add(capture_bytes)
            .ok_or_else(|| SignalError::invalid_input("checked scratch memory overflow"))?;
        super::work::checkpoint(
            work.as_deref_mut(),
            declaration
                .as_slice()
                .len()
                .saturating_add(crate::data::aspect::MAX_ASPECTS + 8),
        )?;
        preparation_budget::claim_vec::<GraphProposalKey>(
            preparation.as_deref_mut(),
            declaration.as_slice().len(),
        )?;
        let effect_count = batch
            .effects()
            .iter()
            .find(|(node, _)| *node == task.node)
            .ok_or_else(|| SignalError::internal("admitted graph task missing effects"))?
            .1
            .len();
        preparation_budget::claim_vec::<GraphProposalKey>(
            preparation.as_deref_mut(),
            effect_count,
        )?;
        if let Some(budget) = preparation.as_deref_mut() {
            // Completed map values outlive the map's own reservation until the
            // epoch owner consumes them, so the request retains their ceiling.
            budget.claim(per_result)?;
        }
        let mut read_keys = declaration
            .as_slice()
            .iter()
            .map(|input| GraphProposalKey::Aspect(input.source, input.aspect))
            .collect::<Vec<_>>();
        read_keys.sort_unstable();
        read_keys.dedup();
        let write_keys = batch
            .effects()
            .iter()
            .find(|(node, _)| *node == task.node)
            .ok_or_else(|| SignalError::internal("admitted graph task missing effects"))?
            .1
            .clone();
        partitions.push(MapPartition {
            identity: PartitionIdentity::new(stage.task_offset() as u64 + index as u64),
            value: GraphWorkItem {
                task_index: index,
                capacity: super::callback::CheckedKernelCapacity {
                    scratch_bytes: per_result,
                    result_bytes,
                },
            },
            read_keys,
            write_keys,
            kernel_scratch_bytes: scratch_bytes,
            max_result_bytes: per_result,
        });
    }
    let identities = partitions
        .iter()
        .map(|partition| partition.identity)
        .collect();
    ExecutionMap::try_from_declared_partitions(identities, partitions)
        .map_err(|_| SignalError::invalid_input("graph epoch access or memory declaration denied"))
}

fn reconcile_prepared_values(
    graph: &SignalGraph,
    tasks: &[EligibleTask],
    validated: Vec<PrevalidatedTask>,
    computed: Vec<PreparedEvaluation>,
    mut work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<Vec<PreparedEvaluation>, SignalError> {
    super::work::checkpoint(work.as_deref_mut(), tasks.len())?;
    preparation_budget::claim_vec::<PreparedEvaluation>(preparation, tasks.len())?;
    let mut computed = computed.into_iter();
    let mut values = Vec::with_capacity(tasks.len());
    for (task, validation) in tasks.iter().zip(validated) {
        match validation {
            PrevalidatedTask::Prepared(prepared) => values.push(prepared),
            PrevalidatedTask::NeedsCompute {
                temporal_ready,
                ready_invalidation,
            } => {
                let mut prepared = computed.next().ok_or_else(|| {
                    SignalError::internal("checked epoch result coverage mismatch")
                })?;
                let declaration = graph
                    .get_contract(task.node)?
                    .execution
                    .bounded_inputs
                    .as_ref()
                    .expect("admitted");
                super::work::checkpoint(
                    work.as_deref_mut(),
                    prepared
                        .dependencies
                        .len()
                        .saturating_mul(declaration.as_slice().len().saturating_add(1)),
                )?;
                if prepared.dependencies.as_slice().iter().any(|edge| {
                    !declaration.contains(edge.source, edge.aspect, edge.scope.as_ref())
                }) {
                    return Err(SignalError::invalid_input(
                        "graph proposal escaped declared inputs",
                    ));
                }
                // Performed attribution is committed by the owner after join,
                // against the observation generation admitted with this epoch.
                if let Some(ready) = ready_invalidation {
                    prepared = crate::logic::invalidation::scheduling::execute_ready(
                        graph,
                        ready,
                        || Ok(prepared),
                    )?;
                }
                if let Some(ready) = temporal_ready {
                    prepared = prepared.with_temporal_eligibility(
                        crate::data::temporal::LoweredTemporalEligibility::Ready(ready),
                    );
                }
                values.push(prepared);
            }
        }
    }
    if computed.next().is_some() {
        return Err(SignalError::internal(
            "checked epoch produced surplus results",
        ));
    }
    Ok(values)
}
