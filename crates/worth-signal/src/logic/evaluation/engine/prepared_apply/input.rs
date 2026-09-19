#[cfg(test)]
use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::NodeEvaluationResult;
use crate::data::reuse::{ReuseBoundaryAuthority, ReuseBoundaryContext};
use crate::data::temporal::LoweredTemporalEligibility;
use crate::logic::evaluation::{DeferralReason, SuppressionReason};
use crate::logic::prepared::{PreparedEvaluation, PreparedEvaluationOutcome};

pub(super) struct PassivePreparedEffect {
    pub(super) result: NodeEvaluationResult,
    pub(super) verdict: crate::logic::evaluation::EvaluationVerdict,
    pub(super) temporal_eligibility: Option<LoweredTemporalEligibility>,
    pub(super) reuse_boundary_authority: ReuseBoundaryAuthority,
    pub(super) reuse_boundary_detail: Option<ReuseBoundaryContext>,
    pub(super) keyed: Option<crate::logic::prepared::PreparedKeyedContext>,
    pub(super) causality: Option<crate::data::trace::CausalityMetadata>,
}

pub(super) fn lower_passive_prepared_effect<R>(
    graph: &SignalGraph,
    node: NodeId,
    prepared: PreparedEvaluation,
    resolve_reuse_boundary_authority: R,
) -> Result<PassivePreparedEffect, SignalError>
where
    R: FnOnce(
        &SignalGraph,
        NodeId,
        Option<&NodeEvaluationResult>,
        Option<&crate::logic::prepared::PreparedKeyedContext>,
    ) -> Result<ReuseBoundaryAuthority, SignalError>,
{
    let PreparedEvaluation {
        outcome,
        keyed,
        trace_data,
        ..
    } = prepared;
    let temporal_eligibility = trace_data.temporal_eligibility;
    let verdict = match outcome {
        PreparedEvaluationOutcome::ValidatedClean => {
            ensure_temporal_outcome_alignment(outcome, temporal_eligibility.as_ref())?;
            crate::logic::evaluation::EvaluationVerdict::Suppressed {
                reason: SuppressionReason::ValidatedClean,
            }
        }
        PreparedEvaluationOutcome::DeferredByInvalidation => {
            ensure_temporal_outcome_alignment(outcome, temporal_eligibility.as_ref())?;
            crate::logic::evaluation::EvaluationVerdict::Deferred {
                reason: DeferralReason::DependencyPending,
            }
        }
        PreparedEvaluationOutcome::DeferredByCondition => {
            ensure_temporal_outcome_alignment(outcome, temporal_eligibility.as_ref())?;
            crate::logic::evaluation::EvaluationVerdict::Deferred {
                reason: if temporal_eligibility.is_some() {
                    DeferralReason::TemporalConditionNotMet
                } else {
                    DeferralReason::ConditionNotMet
                },
            }
        }
        PreparedEvaluationOutcome::RevertedCleanByCondition => {
            ensure_temporal_outcome_alignment(outcome, temporal_eligibility.as_ref())?;
            crate::logic::evaluation::EvaluationVerdict::Suppressed {
                reason: SuppressionReason::ConditionRevertedClean,
            }
        }
        PreparedEvaluationOutcome::Evaluate => {
            return Err(SignalError::internal(
                "passive prepared-effect lowering cannot accept evaluate outcomes",
            ));
        }
    };
    let result = NodeEvaluationResult::from_version(graph.node_aspect_version(node)?);

    Ok(PassivePreparedEffect {
        result,
        verdict,
        temporal_eligibility,
        reuse_boundary_authority: resolve_reuse_boundary_authority(
            graph,
            node,
            None,
            keyed.as_ref(),
        )?,
        reuse_boundary_detail: None,
        keyed,
        causality: trace_data.causality,
    })
}

pub(super) fn ensure_temporal_outcome_alignment(
    outcome: PreparedEvaluationOutcome,
    temporal_eligibility: Option<&LoweredTemporalEligibility>,
) -> Result<(), SignalError> {
    match (outcome, temporal_eligibility) {
        (PreparedEvaluationOutcome::Evaluate, Some(LoweredTemporalEligibility::Deferred(_))) => {
            Err(SignalError::internal(
                "evaluate outcome cannot carry deferred temporal eligibility proof",
            ))
        }
        (
            PreparedEvaluationOutcome::DeferredByCondition,
            Some(LoweredTemporalEligibility::Ready(_)),
        ) => Err(SignalError::internal(
            "deferred outcome cannot carry ready temporal eligibility proof",
        )),
        (
            PreparedEvaluationOutcome::ValidatedClean
            | PreparedEvaluationOutcome::DeferredByInvalidation
            | PreparedEvaluationOutcome::RevertedCleanByCondition,
            Some(_),
        ) => Err(SignalError::internal(
            "passive non-temporal suppression cannot carry temporal eligibility proof",
        )),
        _ => Ok(()),
    }
}

#[cfg(test)]
pub(super) fn apply_prepared_dependencies(
    graph: &mut SignalGraph,
    node: NodeId,
    capture: &PreparedDependencyCapture,
) -> Result<u32, SignalError> {
    let desired = build_prepared_dependency_edges(capture);
    let report = graph.reconcile_dependencies(node, &desired)?;
    Ok(report.added + report.removed)
}

#[cfg(test)]
fn build_prepared_dependency_edges(capture: &PreparedDependencyCapture) -> Vec<DependencyEdge> {
    capture
        .as_slice()
        .iter()
        .map(|dependency| match dependency.scope.clone() {
            Some(scope) => {
                DependencyEdge::with_partition_scope(dependency.source, dependency.aspect, scope)
            }
            None => DependencyEdge::new(dependency.source, dependency.aspect),
        })
        .collect()
}

#[cfg(test)]
use crate::logic::prepared::PreparedDependencyCapture;
