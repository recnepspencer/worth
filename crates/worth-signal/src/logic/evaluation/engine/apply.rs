mod dependency_inputs;
mod effect_lowering;
mod mutation;
mod telemetry;
mod verdict;

use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::NodeEvaluationResult;
use crate::data::reuse::{ReuseBoundaryAuthority, ReuseBoundaryContext, ReuseCertificationRecord};
use crate::logic::evaluation::{
    EffectDependencyInputs, EvaluationVerdict, PreparedApplyResult, PreviousArtifactWarmSnapshot,
};
use crate::logic::prepared::PreparedKeyedContext;

use super::metadata::EvaluationExecutionMetadata;

pub(crate) use dependency_inputs::collect_effect_dependency_inputs_iter;
#[cfg(feature = "parallel")]
pub(crate) use effect_lowering::build_evaluation_effect;
pub(crate) use verdict::provisional_evaluated_verdict;

pub(crate) fn apply_effect_with_policy_and_condition(
    graph: &mut SignalGraph,
    node: NodeId,
    result: NodeEvaluationResult,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
    verdict: EvaluationVerdict,
    recomputed: bool,
    reuse_boundary_authority: ReuseBoundaryAuthority,
    reuse_boundary_detail: Option<ReuseBoundaryContext>,
    keyed_context: Option<PreparedKeyedContext>,
    causality: Option<crate::data::trace::CausalityMetadata>,
    reuse_certification: Option<ReuseCertificationRecord>,
    dependency_inputs: Option<EffectDependencyInputs>,
    defer_snapshot_commit: bool,
    previous_artifact_warm: Option<PreviousArtifactWarmSnapshot>,
    work: &mut super::work::EvaluationWork<'_>,
) -> Result<PreparedApplyResult, SignalError> {
    let dependency_inputs =
        dependency_inputs::resolve_effect_dependency_inputs(graph, node, dependency_inputs, work)?;
    let effect = effect_lowering::build_evaluation_effect(
        node,
        result,
        execution_metadata,
        verdict,
        recomputed,
        reuse_boundary_authority,
        reuse_boundary_detail,
        keyed_context,
        causality,
        reuse_certification,
        dependency_inputs,
        previous_artifact_warm,
    );
    let output_equivalence = effect_lowering::resolve_output_equivalence(graph, node)?;
    let (report, pending_snapshot) = mutation::apply_evaluation_effect(
        graph,
        effect,
        output_equivalence,
        comparator_resolver,
        defer_snapshot_commit,
        work,
    )?;
    Ok(PreparedApplyResult {
        dependency_updates: 0,
        report,
        pending_snapshot,
        temporal_eligibility: None,
    })
}
