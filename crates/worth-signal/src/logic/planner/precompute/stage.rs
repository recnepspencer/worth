use super::super::execution::{AdmittedEpoch, EpochMetadata};
use super::super::types::{PlanSummary, ResolvedSignalPlannerPolicy};
use super::callback::SignalPrecompute;
use super::read_preparation::PreparedEpoch;
use super::{StageExecutionData, TemporalLoweringContext};
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::{invalidation::progression::DisjointGraphBatch, SingleConsumer};
use crate::data::request_preparation::SignalPreparationBudget;
use worth_execution::MapKernelContext;

pub(in crate::logic::planner) enum PreparedStageEpoch<'tasks, 'request, 'authority> {
    Checked {
        metadata: EpochMetadata<'tasks>,
        proposals: StageExecutionData,
        batch: DisjointGraphBatch,
        request: worth_execution::ExecutionRequest<'request, 'authority>,
        apply: super::graph_batch::CheckedApplyCapacity,
        prepared_map:
            crate::logic::planner::apply::stage::PreparedSignalApplyMap<'request, 'authority>,
        candidates: crate::data::graph::PreparedCandidateEpoch<'request, 'authority>,
        reports: Vec<worth_foundational::ExecutionReport>,
    },
    LegacySerial {
        metadata: EpochMetadata<'tasks>,
        proposals: StageExecutionData,
    },
}

impl PreparedStageEpoch<'_, '_, '_> {
    pub(in crate::logic::planner) fn len(&self) -> usize {
        match self {
            Self::Checked { proposals, .. } | Self::LegacySerial { proposals, .. } => {
                proposals.len()
            }
        }
    }

    pub(in crate::logic::planner) fn reports(&self) -> &[worth_foundational::ExecutionReport] {
        match self {
            Self::Checked { reports, .. } => reports,
            Self::LegacySerial { .. } => &[],
        }
    }

    pub(in crate::logic::planner) fn metadata(&self) -> &EpochMetadata<'_> {
        match self {
            Self::Checked { metadata, .. } | Self::LegacySerial { metadata, .. } => metadata,
        }
    }
}

pub(in crate::logic::planner) struct StagePrecomputeResult<'tasks, 'request, 'authority> {
    pub(in crate::logic::planner) prepared: PreparedStageEpoch<'tasks, 'request, 'authority>,
    pub(in crate::logic::planner) snapshot_nanos: u128,
    pub(in crate::logic::planner) precompute_nanos: u128,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::logic::planner) fn perform_stage_precompute<'tasks, 'request, 'authority>(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    stage: AdmittedEpoch<'tasks, 'request, 'authority>,
    precompute: &impl SignalPrecompute,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    temporal_lowering: &TemporalLoweringContext,
    policy: &ResolvedSignalPlannerPolicy,
    request_work: Option<&mut MapKernelContext<'_, '_>>,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<StagePrecomputeResult<'tasks, 'request, 'authority>, SignalError> {
    let started = crate::clock::RuntimeInstant::now();
    let stage_index = stage.index();
    let prepared = super::read_preparation::prepare_epoch(
        graph,
        stage,
        precompute,
        comparator_resolver,
        temporal_lowering,
        policy,
        request_work,
        preparation,
    )
    .inspect_err(|error| {
        super::reporting::record_stage_precompute_failure(graph, summary, stage_index, error);
    })?;
    let precompute_nanos = started.elapsed().as_nanos();
    let prepared = match prepared {
        PreparedEpoch::Checked {
            metadata,
            values,
            batch,
            request,
            apply,
            prepared_map,
            candidates,
            reports,
        } => PreparedStageEpoch::Checked {
            metadata,
            proposals: StageExecutionData::Prepared(SingleConsumer::new(values)),
            batch,
            request,
            apply,
            prepared_map,
            candidates,
            reports,
        },
        PreparedEpoch::LegacySerial { metadata, values } => PreparedStageEpoch::LegacySerial {
            metadata,
            proposals: StageExecutionData::Prepared(SingleConsumer::new(values)),
        },
    };
    super::reporting::record_stage_precompute_telemetry(
        graph,
        prepared.len(),
        0,
        precompute_nanos,
        prepared.reports(),
    );
    Ok(StagePrecomputeResult {
        prepared,
        snapshot_nanos: 0,
        precompute_nanos,
    })
}
