use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::NodeEvaluationResult;
use crate::data::reuse::{
    ReuseBasis, ReuseBoundaryAuthority, ReuseBoundaryContext, ReuseCertificationRecord,
};
use crate::data::temporal::LoweredTemporalEligibility;
use crate::logic::evaluation::{
    EffectDependencyInputs, EvaluationVerdict, EvaluationWork, PreparedApplyResult,
    PreviousArtifactWarmSnapshot,
};
use crate::logic::prepared::{PreparedEvaluation, PreparedEvaluationOutcome};

use super::super::apply::{apply_effect_with_policy_and_condition, provisional_evaluated_verdict};
use super::super::metadata::EvaluationExecutionMetadata;
use super::input::{
    apply_prepared_dependencies, ensure_temporal_outcome_alignment, lower_passive_prepared_effect,
};
use super::reuse_admission::{resolve_evaluated_reuse_admission, EvaluatedReuseAdmission};

pub(crate) fn apply_prepared_evaluation_with_policy(
    graph: &mut SignalGraph,
    node: NodeId,
    prepared: PreparedEvaluation,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
) -> Result<PreparedApplyResult, SignalError> {
    let dependency_updates = apply_prepared_dependencies(graph, node, &prepared.dependencies)?;
    apply_prepared_evaluation_after_dependencies_with_policy(
        graph,
        node,
        prepared,
        comparator_resolver,
        execution_metadata,
        dependency_updates,
        None,
        false,
        &mut EvaluationWork::Ordinary,
    )
}

pub(crate) fn apply_prepared_evaluation_after_dependencies_with_policy(
    graph: &mut SignalGraph,
    node: NodeId,
    prepared: PreparedEvaluation,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
    dependency_updates: u32,
    dependency_inputs: Option<EffectDependencyInputs>,
    defer_snapshot_commit: bool,
    work: &mut EvaluationWork<'_>,
) -> Result<PreparedApplyResult, SignalError> {
    if !matches!(prepared.outcome, PreparedEvaluationOutcome::Evaluate) {
        return apply_passive_prepared_evaluation(
            graph,
            node,
            prepared,
            comparator_resolver,
            execution_metadata,
            dependency_updates,
            dependency_inputs,
            defer_snapshot_commit,
            work,
        );
    }

    apply_evaluated_prepared_evaluation(
        graph,
        node,
        prepared,
        comparator_resolver,
        execution_metadata,
        dependency_updates,
        dependency_inputs,
        defer_snapshot_commit,
        work,
    )
}

fn apply_passive_prepared_evaluation(
    graph: &mut SignalGraph,
    node: NodeId,
    prepared: PreparedEvaluation,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
    dependency_updates: u32,
    dependency_inputs: Option<EffectDependencyInputs>,
    defer_snapshot_commit: bool,
    work: &mut EvaluationWork<'_>,
) -> Result<PreparedApplyResult, SignalError> {
    let passive =
        lower_passive_prepared_effect(graph, node, prepared, |graph, node, result, keyed| {
            crate::logic::evaluation::resolve_reuse_boundary_authority(
                graph,
                node,
                comparator_resolver,
                result,
                keyed,
            )
        })?;
    let mut apply_result = apply_effect_with_policy_and_condition(
        graph,
        node,
        passive.result,
        comparator_resolver,
        execution_metadata,
        passive.verdict,
        false,
        passive.reuse_boundary_authority,
        passive.reuse_boundary_detail,
        passive.keyed,
        passive.causality,
        None,
        dependency_inputs,
        defer_snapshot_commit,
        None,
        work,
    )?;
    apply_result.dependency_updates = dependency_updates;
    apply_result.report.temporal_eligibility = passive.temporal_eligibility.clone();
    apply_result.temporal_eligibility = passive.temporal_eligibility;
    Ok(apply_result)
}

fn apply_evaluated_prepared_evaluation(
    graph: &mut SignalGraph,
    node: NodeId,
    prepared: PreparedEvaluation,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
    dependency_updates: u32,
    dependency_inputs: Option<EffectDependencyInputs>,
    defer_snapshot_commit: bool,
    work: &mut EvaluationWork<'_>,
) -> Result<PreparedApplyResult, SignalError> {
    let application = build_evaluated_prepared_application(
        graph,
        node,
        prepared,
        comparator_resolver,
        execution_metadata,
    )?;
    let mut apply_result = apply_effect_with_policy_and_condition(
        graph,
        node,
        application.result,
        comparator_resolver,
        Some(&application.metadata),
        application.verdict,
        application.recomputed,
        application.reuse_boundary_authority,
        application.reuse_boundary_detail,
        application.keyed,
        application.causality,
        application.reuse_certification,
        dependency_inputs,
        defer_snapshot_commit,
        application.previous_artifact_warm,
        work,
    )?;
    apply_result.dependency_updates = dependency_updates;
    apply_result.report.temporal_eligibility = application.temporal_eligibility.clone();
    apply_result.temporal_eligibility = application.temporal_eligibility;
    Ok(apply_result)
}

struct EvaluatedPreparedApplication {
    result: NodeEvaluationResult,
    metadata: EvaluationExecutionMetadata,
    verdict: EvaluationVerdict,
    recomputed: bool,
    reuse_boundary_authority: ReuseBoundaryAuthority,
    reuse_boundary_detail: Option<ReuseBoundaryContext>,
    keyed: Option<crate::logic::prepared::PreparedKeyedContext>,
    causality: Option<crate::data::trace::CausalityMetadata>,
    reuse_certification: Option<ReuseCertificationRecord>,
    previous_artifact_warm: Option<PreviousArtifactWarmSnapshot>,
    temporal_eligibility: Option<LoweredTemporalEligibility>,
}

fn build_evaluated_prepared_application(
    graph: &mut SignalGraph,
    node: NodeId,
    prepared: PreparedEvaluation,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
) -> Result<EvaluatedPreparedApplication, SignalError> {
    let PreparedEvaluation {
        mut result,
        trace_data,
        outcome,
        origin,
        keyed,
        ..
    } = prepared;
    ensure_temporal_outcome_alignment(outcome, trace_data.temporal_eligibility.as_ref())?;
    result.labels.extend(trace_data.labels);
    let temporal_eligibility = trace_data.temporal_eligibility;
    let causality = trace_data.causality;
    let admission = resolve_evaluated_reuse_admission(
        graph,
        node,
        &result,
        keyed.as_ref(),
        origin,
        execution_metadata,
        comparator_resolver,
    )?;
    let metadata = metadata_for_evaluated_application(&admission, execution_metadata);
    let verdict = provisional_evaluated_verdict();
    Ok(EvaluatedPreparedApplication {
        result,
        metadata,
        verdict,
        recomputed: admission.decision.recomputed,
        reuse_boundary_authority: admission.current_boundary_authority,
        reuse_boundary_detail: admission.current_boundary_detail,
        keyed,
        causality,
        reuse_certification: admission.certification,
        previous_artifact_warm: admission.previous_artifact_warm,
        temporal_eligibility,
    })
}

fn metadata_for_evaluated_application(
    admission: &EvaluatedReuseAdmission,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
) -> EvaluationExecutionMetadata {
    let lowered_reuse_basis = admission
        .decision
        .strategy
        .map(|strategy| {
            ReuseBasis::from_boundary_authority(
                strategy,
                admission.decision.source,
                admission.decision.crossing,
                &admission.current_boundary_authority,
            )
        })
        .unwrap_or_else(ReuseBasis::fresh_compute);
    execution_metadata
        .cloned()
        .unwrap_or(EvaluationExecutionMetadata {
            keyed: None,
            memoized_origin: admission.decision.memoized_origin,
            reuse_basis: lowered_reuse_basis,
            reuse_origin: admission.decision.origin,
        })
}
