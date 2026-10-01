use crate::data::comparator::VersionComparatorPolicy;
use crate::data::dependency::CanonicalDependencies;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::host_computed::{admit_or_error, HostComputedApiFamily};
use crate::data::node::{AuthorityPolicy, PathClass};
use crate::data::performance::{ResolvedExecutionStrategy, ResolvedMaintenanceStrategy};
use crate::logic::planner::precompute::{PreparedTaskPatch, StageExecutionData};
use crate::logic::planner::semantic::StageSemanticIdentity;
use crate::logic::planner::types::{
    EligibleTask, LoweredStagePlan, LoweredTask, LoweredTaskExecution, ResolvedSignalPlannerPolicy,
};
use crate::logic::prepared::{PreparedEvaluationOrigin, PreparedEvaluationOutcome};

use super::super::serial_batch::LoweredSerialStage;
use super::footprint::{
    build_apply_footprint, build_lowered_dirty_delta, build_touched_scope_summary, structural_delta,
};
use super::strategy::build_lowered_apply_plan;
use crate::logic::planner::precompute::graph_batch::CheckedApplyCapacity;

pub(super) enum LoweredStageExecutionForm {
    Serial(LoweredSerialStage),
    Generic(LoweredStagePlan),
}

pub(super) fn build_legacy_stage_execution_form(
    graph: &mut SignalGraph,
    stage_index: u32,
    stage_tasks: &[EligibleTask],
    stage_execution: StageExecutionData,
    stage_identities: &[StageSemanticIdentity],
) -> Result<LoweredSerialStage, SignalError> {
    let prepared_patches = stage_execution.into_patches(stage_tasks);
    let resolved_policy = graph.resolved_performance_policy();
    LoweredSerialStage::from_prepared_patches(
        graph,
        stage_index,
        stage_tasks,
        prepared_patches,
        resolved_policy.maintenance_strategy,
        resolved_policy.authority_policy,
        stage_identities,
    )
}

pub(super) fn build_checked_stage_execution_form(
    graph: &mut SignalGraph,
    stage_index: u32,
    stage_tasks: &[EligibleTask],
    stage_execution: StageExecutionData,
    apply: &mut CheckedApplyCapacity,
    policy: &ResolvedSignalPlannerPolicy,
) -> Result<LoweredStagePlan, SignalError> {
    let prepared_patches = stage_execution.into_patches(stage_tasks);
    let resolved_policy = graph.resolved_performance_policy();
    if apply.members().len() != stage_tasks.len() {
        return Err(SignalError::internal(
            "checked apply capacity omitted a selected task",
        ));
    }
    let lowered_tasks = prepared_patches
        .into_iter()
        .map(|patch| {
            let comparator_policy = apply.take_policy(patch.task_index)?;
            lower_task_patch(graph, patch, comparator_policy)
        })
        .collect::<Result<Vec<_>, SignalError>>()?;
    let lowered_apply_plan = build_lowered_apply_plan(&lowered_tasks, policy);
    let dirty_delta = build_lowered_dirty_delta(&lowered_tasks);
    let touched_scope = build_touched_scope_summary(&lowered_tasks);
    let authority_policy = lowered_tasks
        .iter()
        .find(|task| matches!(task.authority_policy(), AuthorityPolicy::AuthoritativeOnly))
        .map(|task| task.authority_policy())
        .unwrap_or(resolved_policy.authority_policy);
    Ok(LoweredStagePlan::new(
        stage_index,
        lowered_tasks,
        lowered_apply_plan,
        structural_delta(dirty_delta, touched_scope),
        resolved_policy.execution_strategy,
        resolved_policy.maintenance_strategy,
        authority_policy,
    ))
}

pub(super) fn validate_lowered_stage_plan(lowered: &LoweredStagePlan) {
    let rich_task_count = lowered
        .tasks()
        .iter()
        .filter(|task| matches!(task.path_class(), PathClass::Rich))
        .count();
    let authoritative_task_count = lowered
        .tasks()
        .iter()
        .filter(|task| matches!(task.authority_policy(), AuthorityPolicy::AuthoritativeOnly))
        .count();
    let recomputed_task_count = lowered
        .tasks()
        .iter()
        .filter(|task| task.execution().recomputed())
        .count();

    debug_assert!(
        lowered.task_count() == lowered.tasks().len(),
        "lowered task count must match staged task collection"
    );
    debug_assert!(
        lowered.dirty_delta().is_empty() || !lowered.tasks().is_empty(),
        "structural delta should only be populated for non-empty lowered stages"
    );
    debug_assert!(
        !matches!(
            lowered.execution_strategy(),
            ResolvedExecutionStrategy::FullGraphPass
        ) || lowered.tasks().is_empty()
            || !lowered.apply_groups().is_empty(),
        "full-graph execution stages must still lower into apply groups"
    );
    debug_assert!(
        !matches!(
            lowered.maintenance_strategy(),
            ResolvedMaintenanceStrategy::Rebuild
        ) || lowered.dirty_delta().dirty.is_some(),
        "rebuild-oriented stages must carry a narrowed dirty delta"
    );
    debug_assert!(
        !matches!(
            lowered.authority_policy(),
            AuthorityPolicy::AuthoritativeOnly
        ) || authoritative_task_count > 0,
        "authoritative lowered stages must include authoritative tasks"
    );
    debug_assert!(
        rich_task_count <= lowered.tasks().len(),
        "rich-path accounting must remain bounded by lowered tasks"
    );
    debug_assert!(
        recomputed_task_count <= lowered.tasks().len(),
        "recomputed-task accounting must remain bounded by lowered tasks"
    );
}

fn lower_task_patch(
    graph: &mut SignalGraph,
    patch: PreparedTaskPatch,
    comparator_policy: VersionComparatorPolicy,
) -> Result<LoweredTask, SignalError> {
    // The admitted graph epoch binds the current dependency revision. A refresh
    // here would mutate that binding before the whole proposal is preflighted.
    let current_dependencies =
        CanonicalDependencies::from_slice(graph.current_runtime_dependencies_of(patch.node)?);
    let admitted = {
        let mut telemetry_guard = graph.telemetry_mut();
        let telemetry = telemetry_guard.as_deref_mut();
        admit_or_error(
            HostComputedApiFamily::CorePreparedEvaluation,
            patch.node,
            current_dependencies.as_slice(),
            patch.prepared,
            telemetry,
        )?
    };
    let (prepared, _admitted_reads, dependency_patch) = admitted.into_parts();
    let next_dependencies = CanonicalDependencies::from_slice(dependency_patch.next_dependencies());
    let before_state = graph.get_state(patch.node)?;
    let before_artifact_state = graph.node_runtime_artifact_finalize_image(patch.node)?;
    let contract = graph.get_contract(patch.node)?;
    let recomputed = matches!(prepared.outcome, PreparedEvaluationOutcome::Evaluate)
        && !matches!(prepared.origin, PreparedEvaluationOrigin::MemoizedReuse);
    let partition_aware = !prepared.result.changed_regions.is_empty();
    let rewiring = super::super::lowering_support::rewiring_summary_from_lowered_edges(
        current_dependencies.as_slice(),
        next_dependencies.as_slice(),
    );
    let footprint = build_apply_footprint(patch.node, &current_dependencies, &next_dependencies);
    let dependency_updates = super::super::lowering_support::count_dependency_updates(
        current_dependencies.as_slice(),
        next_dependencies.as_slice(),
    );

    Ok(LoweredTask::new(
        patch.task_index,
        patch.node,
        contract.semantics.produces,
        next_dependencies,
        comparator_policy,
        contract.execution.path_class,
        contract.authority.policy,
        footprint,
        LoweredTaskExecution::new(
            prepared,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
        ),
    ))
}
