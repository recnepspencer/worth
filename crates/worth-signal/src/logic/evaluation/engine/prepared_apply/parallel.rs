use crate::data::comparator::VersionComparatorPolicy;
use crate::data::error::SignalError;
use crate::data::graph::{ApplyCommitPacket, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::reuse::{ReuseBoundaryEvidence, ReuseBoundaryFailure};
use crate::logic::evaluation::{EffectDependencyInputs, PreviousArtifactWarmSnapshot};
use crate::logic::prepared::{PreparedEvaluation, PreparedEvaluationOutcome};

use super::super::apply::{build_evaluation_effect, provisional_evaluated_verdict};
use super::super::metadata::EvaluationExecutionMetadata;
use super::admission::{
    format_reuse_boundary_evidence, hydrate_reuse_boundary_evidence,
    resolve_effect_reuse_boundary_with_policy,
};
use super::input::{ensure_temporal_outcome_alignment, lower_passive_prepared_effect};

#[derive(Debug)]
pub(crate) enum ApplyCommitBuildError {
    Signal(SignalError),
    ReuseBoundary {
        error: SignalError,
        failure: ReuseBoundaryFailure,
    },
}

impl ApplyCommitBuildError {
    pub(crate) fn into_signal(self) -> SignalError {
        match self {
            Self::Signal(error) => error,
            Self::ReuseBoundary { error, .. } => error,
        }
    }

    pub(crate) fn reuse_failure(&self) -> Option<ReuseBoundaryFailure> {
        match self {
            Self::Signal(_) => None,
            Self::ReuseBoundary { failure, .. } => Some(*failure),
        }
    }
}

impl From<SignalError> for ApplyCommitBuildError {
    fn from(error: SignalError) -> Self {
        Self::Signal(error)
    }
}

pub(crate) fn build_prepared_apply_commit_packet(
    graph: &SignalGraph,
    node: NodeId,
    prepared: PreparedEvaluation,
    comparator_policy: VersionComparatorPolicy,
    execution_metadata: Option<&EvaluationExecutionMetadata>,
    _dependency_updates: u32,
    dependency_inputs: EffectDependencyInputs,
    defer_snapshot_commit: bool,
) -> Result<ApplyCommitPacket, ApplyCommitBuildError> {
    if !matches!(prepared.outcome, PreparedEvaluationOutcome::Evaluate) {
        let passive =
            lower_passive_prepared_effect(graph, node, prepared, |graph, node, result, keyed| {
                crate::logic::evaluation::resolve_reuse_boundary_authority_with_policy(
                    graph,
                    node,
                    comparator_policy.clone(),
                    result,
                    keyed,
                )
            })?;
        let effect = build_evaluation_effect(
            node,
            passive.result,
            execution_metadata,
            passive.verdict,
            false,
            passive.reuse_boundary_authority,
            passive.reuse_boundary_detail,
            passive.keyed,
            passive.causality,
            None,
            dependency_inputs,
            None,
        );
        return graph
            .build_apply_commit_packet(
                effect,
                graph.node_eval_config(node)?.output_equivalence.clone(),
                defer_snapshot_commit,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary,
            )
            .map_err(ApplyCommitBuildError::Signal);
    }

    let packet = match prepared.outcome {
        PreparedEvaluationOutcome::Evaluate => {
            ensure_temporal_outcome_alignment(
                prepared.outcome,
                prepared.trace_data.temporal_eligibility.as_ref(),
            )?;
            let mut result = prepared.result;
            result.labels.extend(prepared.trace_data.labels);
            let reuse_decision = crate::logic::evaluation::resolve_prepared_reuse_decision(
                prepared.origin,
                execution_metadata,
            );
            let reuse_contract = graph.node_eval_config(node)?.contract.reuse.clone();
            let previous_artifact_warm = graph
                .node_runtime_artifact_reuse_boundary_snapshot(node)?
                .map(|trace| PreviousArtifactWarmSnapshot {
                    output_identity: trace.output_identity,
                    continuity_token: trace.continuity_token,
                    reuse_boundary_authority: trace.reuse_boundary_authority,
                });
            let previous_reuse_boundary_authority = previous_artifact_warm
                .as_ref()
                .and_then(|trace| trace.reuse_boundary_authority.clone());
            let (current_reuse_boundary_authority, current_reuse_boundary_detail) =
                resolve_effect_reuse_boundary_with_policy(
                    graph,
                    node,
                    comparator_policy.clone(),
                    Some(&result),
                    prepared.keyed.as_ref(),
                    reuse_decision.strategy,
                    previous_reuse_boundary_authority.as_ref(),
                )?;
            let admission_boundary_evidence =
                hydrate_reuse_boundary_evidence(ReuseBoundaryEvidence {
                    current: current_reuse_boundary_authority.clone(),
                    previous: previous_reuse_boundary_authority.or_else(|| {
                        matches!(
                            reuse_decision.strategy,
                            Some(crate::data::reuse::ReuseStrategy::CrossIdentityPersistentMatch)
                                | Some(crate::data::reuse::ReuseStrategy::PartialArtifactSplicing)
                        )
                        .then_some(current_reuse_boundary_authority.clone())
                    }),
                });
            let reuse_certification = crate::logic::evaluation::certify_reuse_decision(
                &reuse_contract,
                &reuse_decision,
                &admission_boundary_evidence,
            )
            .map_err(|failure| ApplyCommitBuildError::ReuseBoundary {
                error: SignalError::invalid_input(format!(
                    "reuse certification failed for {node}: {:?}; {}",
                    failure.failure,
                    format_reuse_boundary_evidence(&admission_boundary_evidence)
                )),
                failure: failure.failure,
            })?;
            let lowered_reuse_basis = reuse_decision
                .strategy
                .map(|strategy| {
                    crate::data::reuse::ReuseBasis::from_boundary_authority(
                        strategy,
                        reuse_decision.source,
                        reuse_decision.crossing,
                        &current_reuse_boundary_authority,
                    )
                })
                .unwrap_or_else(crate::data::reuse::ReuseBasis::fresh_compute);
            let synthesized_metadata = EvaluationExecutionMetadata {
                keyed: None,
                memoized_origin: reuse_decision.memoized_origin,
                reuse_basis: lowered_reuse_basis,
                reuse_origin: reuse_decision.origin,
            };
            let metadata = execution_metadata.unwrap_or(&synthesized_metadata);
            let verdict = provisional_evaluated_verdict();
            let effect = build_evaluation_effect(
                node,
                result,
                Some(metadata),
                verdict,
                reuse_decision.recomputed,
                current_reuse_boundary_authority,
                current_reuse_boundary_detail,
                prepared.keyed,
                prepared.trace_data.causality,
                reuse_certification,
                dependency_inputs,
                previous_artifact_warm,
            );
            graph
                .build_apply_commit_packet(
                    effect,
                    graph.node_eval_config(node)?.output_equivalence.clone(),
                    defer_snapshot_commit,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary,
                )
                .map_err(ApplyCommitBuildError::Signal)?
        }
        PreparedEvaluationOutcome::ValidatedClean
        | PreparedEvaluationOutcome::DeferredByInvalidation
        | PreparedEvaluationOutcome::DeferredByCondition
        | PreparedEvaluationOutcome::RevertedCleanByCondition => {
            return Err(ApplyCommitBuildError::Signal(SignalError::internal(
                "passive prepared outcomes must be lowered before entering the evaluate branch",
            )))
        }
    };

    Ok(packet)
}
