use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::{SignalError, SignalPublicationProgress};
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use worth_execution::MapKernelContext;

use super::super::precompute::callback::SignalPrecompute;
use super::super::types::{
    EligibleTask, ExecutionReport, PlanSummary, ResolvedSignalPlannerPolicy,
};
use super::super::TemporalLoweringContext;
use super::context::ExecutionContext;
use super::stage::execute_stage;
use crate::data::request_preparation::SignalPreparationBudget;

#[derive(Clone, Copy)]
pub(crate) struct PlannedStageSlice<'a> {
    pub(crate) index: u32,
    pub(crate) task_offset: usize,
    pub(crate) tasks: &'a [EligibleTask],
}

#[derive(Clone, Copy)]
pub(crate) struct EpochMetadata<'tasks> {
    stage_index: u32,
    tasks: &'tasks [EligibleTask],
}

impl<'tasks> EpochMetadata<'tasks> {
    pub(crate) fn index(&self) -> u32 {
        self.stage_index
    }
    pub(crate) fn tasks(&self) -> &'tasks [EligibleTask] {
        self.tasks
    }
}

pub(crate) struct CheckedAdmittedEpoch<'tasks, 'request, 'authority> {
    stage_index: u32,
    selection: crate::logic::planner::precompute::graph_batch::CheckedEpochAdmission<'tasks>,
    request: worth_execution::ExecutionRequest<'request, 'authority>,
    readiness_epoch: crate::data::proof::invalidation::progression::InvalidationReadinessEpoch,
}

impl<'tasks, 'request, 'authority> CheckedAdmittedEpoch<'tasks, 'request, 'authority> {
    pub(crate) fn tasks(&self) -> &[EligibleTask] {
        self.selection.tasks()
    }

    pub(crate) fn task_offset(&self) -> usize {
        self.selection.task_offset()
    }

    pub(crate) fn result_grant_bytes(&self) -> u64 {
        self.selection.result_grant_bytes()
    }

    pub(in crate::logic::planner) fn apply_capacity(
        &self,
    ) -> &crate::logic::planner::precompute::graph_batch::CheckedApplyCapacity {
        self.selection.apply_capacity()
    }

    pub(crate) fn request(&self) -> worth_execution::ExecutionRequest<'request, 'authority> {
        self.request
    }

    pub(crate) fn into_prepared_parts(
        self,
    ) -> (
        EpochMetadata<'tasks>,
        worth_execution::ExecutionRequest<'request, 'authority>,
        crate::logic::planner::precompute::graph_batch::CheckedApplyCapacity,
    ) {
        let (tasks, apply) = self.selection.into_parts();
        (
            EpochMetadata {
                stage_index: self.stage_index,
                tasks,
            },
            self.request,
            apply,
        )
    }
}

pub(crate) struct LegacySerialEpoch<'tasks> {
    stage_index: u32,
    task_offset: usize,
    tasks: &'tasks [EligibleTask],
    readiness_epoch: crate::data::proof::invalidation::progression::InvalidationReadinessEpoch,
}

impl LegacySerialEpoch<'_> {
    pub(crate) fn tasks(&self) -> &[EligibleTask] {
        self.tasks
    }
}

impl<'tasks> LegacySerialEpoch<'tasks> {
    pub(crate) fn into_metadata(self) -> EpochMetadata<'tasks> {
        EpochMetadata {
            stage_index: self.stage_index,
            tasks: self.tasks,
        }
    }
}

pub(crate) enum AdmittedEpoch<'tasks, 'request, 'authority> {
    Checked(CheckedAdmittedEpoch<'tasks, 'request, 'authority>),
    LegacySerial(LegacySerialEpoch<'tasks>),
}

impl AdmittedEpoch<'_, '_, '_> {
    pub(crate) fn index(&self) -> u32 {
        match self {
            Self::Checked(epoch) => epoch.stage_index,
            Self::LegacySerial(epoch) => epoch.stage_index,
        }
    }

    pub(crate) fn task_offset(&self) -> usize {
        match self {
            Self::Checked(epoch) => epoch.task_offset(),
            Self::LegacySerial(epoch) => epoch.task_offset,
        }
    }

    pub(crate) fn tasks(&self) -> &[EligibleTask] {
        match self {
            Self::Checked(epoch) => epoch.tasks(),
            Self::LegacySerial(epoch) => epoch.tasks,
        }
    }

    pub(crate) fn readiness_epoch(
        &self,
    ) -> crate::data::proof::invalidation::progression::InvalidationReadinessEpoch {
        match self {
            Self::Checked(epoch) => epoch.readiness_epoch,
            Self::LegacySerial(epoch) => epoch.readiness_epoch,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_stage_slices<'a, P: SignalPrecompute>(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    stage_count: usize,
    maybe_stale_validation_tasks: u64,
    stages: impl IntoIterator<Item = PlannedStageSlice<'a>>,
    precompute: &P,
    plan_summary: crate::diagnostics::summary::EvaluationPlanSummary,
    first_target: Option<NodeId>,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    temporal_lowering: TemporalLoweringContext,
    request: worth_execution::ExecutionRequest<'_, '_>,
    policy: ResolvedSignalPlannerPolicy,
    request_work: &mut MapKernelContext<'_, '_>,
    preparation: &mut SignalPreparationBudget,
    progress: &mut SignalPublicationProgress,
) -> Result<ExecutionReport, SignalError> {
    let mut context = ExecutionContext::new(
        graph,
        summary,
        stage_count,
        maybe_stale_validation_tasks,
        plan_summary,
        first_target,
        precompute,
        comparator_resolver,
        temporal_lowering,
        request,
        policy,
        request_work,
        preparation,
    );
    for stage in stages {
        let mut offset = 0;
        let mut stage_readiness_epoch = None;
        while offset < stage.tasks.len() {
            // The prior epoch's preparation has been consumed and published before
            // the next admission. Keep the request's planning and report storage.
            let frame = context.preparation.checkpoint();
            let width = context
                .request
                .in_scope(|_scoped_lease| {
                    let admission = super::super::precompute::graph_batch::epoch_width(
                        context.graph,
                        &stage.tasks[offset..],
                        stage.task_offset + offset,
                        precompute.allows_bounded_inputs(),
                        context.request,
                        context.comparator_resolver,
                        Some(&mut *context.preparation),
                        Some(&mut *context.request_work),
                    )?;
                    // Resource slices of one planned stage share its semantic readiness
                    // identity. Mint it only after the first slice has been admitted.
                    let readiness_epoch = *stage_readiness_epoch
                        .get_or_insert_with(|| context.graph.begin_invalidation_readiness_epoch());
                    let epoch = match admission {
                        super::super::precompute::graph_batch::EpochAdmission::Checked(
                            selection,
                        ) => {
                            let request = context.request;
                            AdmittedEpoch::Checked(CheckedAdmittedEpoch {
                                stage_index: stage.index,
                                selection,
                                request,
                                readiness_epoch,
                            })
                        }
                        super::super::precompute::graph_batch::EpochAdmission::LegacySerial => {
                            AdmittedEpoch::LegacySerial(LegacySerialEpoch {
                                stage_index: stage.index,
                                task_offset: stage.task_offset + offset,
                                tasks: &stage.tasks[offset..offset + 1],
                                readiness_epoch,
                            })
                        }
                    };
                    let width = epoch.tasks().len();
                    if let Err(error) = execute_stage(&mut context, epoch, progress) {
                        if let SignalError::ExecutionStopped(stop) = &error {
                            progress.stopped_epoch(stop.disposition());
                        }
                        return Err(error);
                    }
                    Ok(width)
                })
                .map_err(SignalError::execution_scope_denied)??;
            progress.complete_epoch(width);
            context.preparation.release(frame);
            offset += width;
        }
    }
    Ok(context.finish())
}
