use super::callback::SignalPrecompute;
use super::eligibility::{prevalidate_stage_tasks, PrevalidatedTask};
use super::TemporalLoweringContext;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::{SignalError, SignalExecutionStop, SignalPublicationDisposition};
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::progression::DisjointGraphBatch;
use crate::data::request_preparation::{self as preparation_budget, SignalPreparationBudget};
use crate::logic::planner::execution::{AdmittedEpoch, EpochMetadata};
use crate::logic::planner::types::{EligibleTask, ResolvedSignalPlannerPolicy};
use crate::logic::prepared::{ExecutionSnapshot, PreparedEvaluation};
use worth_execution::{ExecutionResourceLease, MapKernelContext, MapKernelFailure, MapOutcome};

mod capacity;
mod map_declaration;
pub(super) use capacity::{checked_map_memory_requirement, PrecomputeMapBasis};
use map_declaration::lower_checked_map;

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
            return prepare_legacy_epoch(
                graph,
                legacy,
                validated,
                precompute,
                request_work.expect("serial preparation carries request work"),
                preparation.expect("serial preparation carries its memory owner"),
            );
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
                .map_err(SignalError::execution_admission_denied)?,
        )
    } else {
        None
    };
    let outcome = map.run(serial_lease.as_ref().or(Some(lease)), |item, work| {
        let task = &checked.tasks()[item.task_index];
        work.checkpoint(1).map_err(MapKernelFailure::Stop)?;
        let view = snapshot.read_view(task.node);
        precompute
            .prepare(task.node, &view, work, Some(item.capacity))
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
    request_work: &mut MapKernelContext<'_, '_>,
    preparation: &mut SignalPreparationBudget,
) -> Result<PreparedEpoch<'tasks, 'static, 'static>, SignalError> {
    let tasks = admitted.tasks();
    let snapshot = ExecutionSnapshot::new(graph);
    let mut values = allocate_legacy_values(tasks.len(), preparation)?;
    for (task, validation) in tasks.iter().zip(validated) {
        match validation {
            PrevalidatedTask::Prepared(prepared) => values.push(prepared),
            PrevalidatedTask::NeedsCompute {
                temporal_ready,
                ready_invalidation,
            } => {
                let view = snapshot.read_view(task.node);
                let mut compute = || precompute.prepare(task.node, &view, &mut *request_work, None);
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

fn allocate_legacy_values(
    count: usize,
    preparation: &mut SignalPreparationBudget,
) -> Result<Vec<PreparedEvaluation>, SignalError> {
    preparation.claim_vec::<PreparedEvaluation>(count)?;
    Ok(Vec::with_capacity(count))
}

#[cfg(test)]
#[path = "legacy_memory_tests.rs"]
mod legacy_memory_tests;
