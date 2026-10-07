mod concurrent;
mod concurrent_packets;
mod footprint;
mod lowering;
mod metrics;
mod strategy;

pub(in crate::logic::planner) use concurrent::{prepare_checked_apply_map, PreparedSignalApplyMap};

use crate::clock::RuntimeInstant;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::logic::planner::execution::EpochMetadata;
use crate::logic::planner::precompute::stage::{PreparedStageEpoch, StagePrecomputeResult};
use crate::logic::planner::semantic::{finalize_serial_stage_batch, StageSemanticIdentity};
use crate::logic::planner::types::{
    ExecutionReport, PlanSummary, ResolvedSignalPlannerPolicy, StageExecutionRecord,
};
use worth_execution::{ExecutionResourceLease, MapKernelContext};

use super::serial_batch::{LoweredSerialStage, PreparedSerialStageBatch};
use super::workspace::{StageFinalizeWork, StageScratch};

pub(super) enum ApplyAdmission<'tasks, 'lease, 'authority> {
    Checked {
        metadata: EpochMetadata<'tasks>,
        batch: crate::data::proof::invalidation::progression::DisjointGraphBatch,
        // Keep checked publication within the originating request borrow.
        _lease: &'lease ExecutionResourceLease<'authority>,
        apply: crate::logic::planner::precompute::graph_batch::CheckedApplyCapacity,
        prepared_map: PreparedSignalApplyMap<'authority>,
        candidates: crate::data::graph::PreparedCandidateEpoch<'authority>,
    },
    LegacySerial {
        metadata: EpochMetadata<'tasks>,
    },
}

impl<'tasks> ApplyAdmission<'tasks, '_, '_> {
    fn metadata(&self) -> EpochMetadata<'tasks> {
        match self {
            Self::Checked { metadata, .. } | Self::LegacySerial { metadata } => *metadata,
        }
    }
}

pub(in crate::logic::planner) fn apply_stage<R>(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    precomputed: StagePrecomputeResult<'_, '_, '_>,
    comparator_resolver: &mut R,
    policy: &ResolvedSignalPlannerPolicy,
    stage_identities: &[StageSemanticIdentity],
    report: &mut ExecutionReport,
    stage_record: &mut StageExecutionRecord,
    request_work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<(), SignalError>
where
    R: ComparatorPolicyResolver,
{
    let (proposals, mut admission) = match precomputed.prepared {
        PreparedStageEpoch::Checked {
            metadata,
            proposals,
            batch,
            lease,
            apply,
            prepared_map,
            candidates,
            ..
        } => (
            proposals,
            ApplyAdmission::Checked {
                metadata,
                batch,
                _lease: lease,
                apply,
                prepared_map,
                candidates,
            },
        ),
        PreparedStageEpoch::LegacySerial {
            metadata,
            proposals,
        } => (proposals, ApplyAdmission::LegacySerial { metadata }),
    };
    let lowered = match &mut admission {
        ApplyAdmission::Checked {
            metadata, apply, ..
        } => lowering::LoweredStageExecutionForm::Generic(
            lowering::build_checked_stage_execution_form(
                graph,
                metadata.index(),
                metadata.tasks(),
                proposals,
                apply,
                policy,
            )?,
        ),
        ApplyAdmission::LegacySerial { metadata } => lowering::LoweredStageExecutionForm::Serial(
            lowering::build_legacy_stage_execution_form(
                graph,
                metadata.index(),
                metadata.tasks(),
                proposals,
                stage_identities,
            )?,
        ),
    };
    let metadata = admission.metadata();
    metrics::record_stage_lowering_metrics(graph, &lowered);
    let stage_scratch = run_lowered_apply_pass(
        graph,
        summary,
        lowered,
        comparator_resolver,
        admission,
        policy,
        stage_identities,
        report,
        stage_record,
        request_work,
        preparation,
    )?;
    let (finalize_work, pending_snapshots) = stage_scratch.into_parts();
    publish_pending_snapshots(graph, pending_snapshots)?;
    finalize_stage_results(graph, metadata.tasks(), finalize_work, report, stage_record)
}

fn publish_pending_snapshots(
    graph: &mut SignalGraph,
    pending_snapshots: crate::data::proof::ClassifiedSnapshotBatchCommit,
) -> Result<(), SignalError> {
    if pending_snapshots.is_empty() {
        return Ok(());
    }
    graph.apply_classified_snapshot_batch_commit(pending_snapshots)
}

fn finalize_stage_results(
    graph: &mut SignalGraph,
    tasks: &[crate::logic::planner::types::EligibleTask],
    finalize_work: StageFinalizeWork,
    report: &mut ExecutionReport,
    stage_record: &mut StageExecutionRecord,
) -> Result<(), SignalError> {
    let semantic_finalize_start = RuntimeInstant::now();
    match finalize_work {
        StageFinalizeWork::Serial(batch) => {
            let ready = batch.into_ready_for_finalize()?;
            finalize_serial_stage_batch(graph, ready, report, stage_record)?
                .record_into(report, stage_record);
        }
        StageFinalizeWork::Parallel(batch) => {
            crate::logic::planner::semantic::finalize_stage_batch(
                graph,
                tasks,
                batch.into_inner(),
                report,
                stage_record,
            )?;
        }
    }
    stage_record.semantic_finalize_duration_nanos = semantic_finalize_start.elapsed().as_nanos();
    Ok(())
}

fn run_lowered_apply_pass<R>(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    lowered: lowering::LoweredStageExecutionForm,
    comparator_resolver: &mut R,
    admission: ApplyAdmission<'_, '_, '_>,
    policy: &ResolvedSignalPlannerPolicy,
    stage_identities: &[StageSemanticIdentity],
    report: &mut ExecutionReport,
    stage_record: &mut StageExecutionRecord,
    request_work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
) -> Result<StageScratch, SignalError>
where
    R: ComparatorPolicyResolver,
{
    match lowered {
        lowering::LoweredStageExecutionForm::Serial(lowered) => {
            stage_record.authority_policy = Some(lowered.authority_policy());
            run_serial_lowered_apply_pass(
                graph,
                summary,
                lowered,
                comparator_resolver,
                stage_record,
                request_work.expect("serial apply carries admitted request work"),
                preparation.expect("serial apply carries its memory owner"),
            )
        }
        lowering::LoweredStageExecutionForm::Generic(lowered) => {
            lowering::validate_lowered_stage_plan(&lowered);
            stage_record.authority_policy = Some(lowered.authority_policy());
            let (stage_index, tasks, lowered_apply_plan, ..) = lowered.into_parts();
            let crate::logic::planner::types::LoweredApplyPlan::GroupedConcurrent(plan) =
                lowered_apply_plan
            else {
                return Err(SignalError::internal(
                    "generic stage dispatch received a serial apply plan after serial lowering",
                ));
            };
            let ApplyAdmission::Checked {
                batch,
                apply,
                prepared_map,
                candidates,
                ..
            } = admission
            else {
                return Err(SignalError::internal(
                    "generic apply lacked checked epoch authority",
                ));
            };
            concurrent::run_grouped_concurrent_apply_pass(
                graph,
                summary,
                stage_index,
                tasks,
                plan,
                policy,
                &batch,
                &apply,
                prepared_map,
                candidates,
                comparator_resolver,
                stage_identities,
                report,
                stage_record,
                request_work,
                preparation,
            )
        }
    }
}

fn run_serial_lowered_apply_pass<R>(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    lowered: LoweredSerialStage,
    comparator_resolver: &mut R,
    stage_record: &mut StageExecutionRecord,
    request_work: &mut MapKernelContext<'_, '_>,
    preparation: &mut crate::data::request_preparation::SignalPreparationBudget,
) -> Result<StageScratch, SignalError>
where
    R: ComparatorPolicyResolver,
{
    let prepared = PreparedSerialStageBatch::prepare(graph, lowered, stage_record)?;
    let mut retained = crate::data::retained_storage::RetainedStoragePreparation::new(usize::MAX);
    let mut checkpoint = |units: usize| {
        request_work.checkpoint(units as u64).map_err(|stop| {
            crate::data::retained_storage::RetainedStoragePreparationDenial::ExecutionStopped(
                stop.into(),
            )
        })
    };
    let mut observed = retained.reborrow_with_checkpoint(&mut checkpoint);
    let mut work = crate::logic::evaluation::EvaluationWork::RequestPreparation {
        work: &mut observed,
        memory: preparation,
    };
    let applied = prepared.apply(graph, summary, comparator_resolver, &mut work)?;
    let (applied, pending_snapshots) = applied.split_pending_snapshots();
    Ok(StageScratch::new(
        StageFinalizeWork::Serial(applied),
        pending_snapshots,
    ))
}
