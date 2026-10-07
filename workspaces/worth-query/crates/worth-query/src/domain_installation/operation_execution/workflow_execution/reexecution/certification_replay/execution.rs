use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::WorthQueryBoundDomainOperation;
use crate::runtime::WorthQueryWorkspace;
use worth_proof::TransitionOutcome;
use worth_query_installation::facade::WorthQueryOperationReplayContract;

use super::super::{
    compare_exact_workflow_traces_counted, WorthQueryCompletedWorkflowTrace,
    WorthQueryExecutableDomainOperation, WorthQueryNormalizedWorkflowIntent,
    WorthQueryReplayComparison, WorthQueryReplayDivergence, WorthQueryReplayNoiseContract,
    WorthQueryWorkflowOperation, WorthQueryWorkflowReexecutionStop,
    WorthQueryWorkflowTraceSemantics,
};
use super::{
    denied, enforce_query_replay_comparison, execution,
    WorthQueryCertificationReplayAdmissionDenial, WorthQueryCertificationReplayCounters,
    WorthQueryCertificationReplayOutcome, WorthQueryCertificationReplayResult,
    WorthQueryCertificationReplayStop, WorthQueryReplayBasisRelationship,
};

pub(in crate::domain_installation::operation_execution) fn execute_admitted_replay<
    D: 'static,
    O,
    F: 'static,
    LO: BasisOperationLane,
    LR: BasisOperationLane,
>(
    original: &WorthQueryCompletedWorkflowTrace<D, O, F, LO>,
    bound: WorthQueryBoundDomainOperation<D, O, F, LR>,
    intent: WorthQueryNormalizedWorkflowIntent,
    resources: worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest,
    workspace: &mut WorthQueryWorkspace,
    basis_relationship: WorthQueryReplayBasisRelationship,
    mut counters: WorthQueryCertificationReplayCounters,
    request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryCertificationReplayOutcome<D, O, F, LR>
where
    O: WorthQueryExecutableDomainOperation<D, F, Execution = WorthQueryWorkflowOperation> + 'static,
{
    let noise = match installed_replay_noise_contract(&bound) {
        Ok(noise) => noise,
        Err(denial) => return denied(denial),
    };
    let original_semantics = original.semantics();
    let original_execution_counters = original.exact_counters();
    if !intent_matches_original_trace(&intent, &original_semantics, &mut counters) {
        return denied(
            WorthQueryCertificationReplayAdmissionDenial::IntentDoesNotMatchOriginalTrace,
        );
    }
    let Some(comparator) = bound.workflow_providers().1.replay_comparator() else {
        return denied(WorthQueryCertificationReplayAdmissionDenial::ReplayComparatorUnavailable);
    };
    let admitted = match bound.admit_workflow_resources(resources, workspace) {
        TransitionOutcome::Success(admitted) => admitted,
        TransitionOutcome::Denied(stop) => {
            return TransitionOutcome::Denied(WorthQueryCertificationReplayStop::ResourceAdmission(
                stop,
            ))
        }
        TransitionOutcome::Deferred(stop) => {
            return TransitionOutcome::Deferred(
                WorthQueryCertificationReplayStop::ResourceAdmission(stop),
            )
        }
        TransitionOutcome::Stale(stop) => {
            return TransitionOutcome::Stale(WorthQueryCertificationReplayStop::ResourceAdmission(
                stop,
            ))
        }
        TransitionOutcome::RebindRequired(stop) => {
            return TransitionOutcome::RebindRequired(
                WorthQueryCertificationReplayStop::ResourceAdmission(stop),
            )
        }
        TransitionOutcome::Failed(stop) => {
            return TransitionOutcome::Failed(WorthQueryCertificationReplayStop::ResourceAdmission(
                stop,
            ))
        }
    };
    let replay_trace = match admitted.reexecute(intent.clone(), workspace, request) {
        TransitionOutcome::Success(trace) => trace,
        TransitionOutcome::Denied(stop) => return TransitionOutcome::Denied(execution(stop)),
        TransitionOutcome::Deferred(WorthQueryWorkflowReexecutionStop::ConditionalDeferred {
            stage_identity,
            ..
        }) => {
            return TransitionOutcome::Deferred(
                WorthQueryCertificationReplayStop::SemanticDivergence(
                    WorthQueryReplayDivergence::ConditionalPath {
                        stage: stage_identity,
                    },
                ),
            )
        }
        TransitionOutcome::Deferred(stop) => return TransitionOutcome::Deferred(execution(stop)),
        TransitionOutcome::Stale(stop) => return TransitionOutcome::Stale(execution(stop)),
        TransitionOutcome::RebindRequired(stop) => {
            return TransitionOutcome::RebindRequired(execution(stop))
        }
        TransitionOutcome::Failed(stop) => return TransitionOutcome::Failed(execution(stop)),
    };
    let replay_semantics = replay_trace.semantics();
    let replay_trace_identity = replay_trace.identity().to_owned();
    let replay_execution_counters = replay_trace.exact_counters();
    let closures_converge = original
        .semantic_aspect_dependency_closure()
        .zip(replay_trace.semantic_aspect_dependency_closure())
        .is_some_and(|(original, replay)| original.converges_with(replay));
    let (mandatory_comparison, compared_stages) = if closures_converge {
        compare_exact_workflow_traces_counted(&original_semantics, &replay_semantics, noise)
    } else {
        (
            WorthQueryReplayComparison::Diverged(WorthQueryReplayDivergence::DependencyClosure),
            0,
        )
    };
    counters.semantic_stage_comparisons = compared_stages;
    let comparison = enforce_query_replay_comparison(mandatory_comparison, || {
        comparator.compare(&original_semantics, &replay_semantics, noise)
    });
    let foundational_attachment = super::super::materialize_replay_attachment(
        original.identity(),
        &replay_trace_identity,
        comparison == WorthQueryReplayComparison::Equivalent,
    );
    TransitionOutcome::Success(WorthQueryCertificationReplayResult {
        original_trace_identity: original.identity().to_owned(),
        replay_trace_identity,
        intent,
        basis_relationship,
        original_semantics,
        replay_semantics,
        comparison,
        foundational_attachment,
        original_execution_counters,
        replay_execution_counters,
        counters,
        _operation: std::marker::PhantomData,
    })
}

fn installed_replay_noise_contract<D, O, F, L: BasisOperationLane>(
    bound: &WorthQueryBoundDomainOperation<D, O, F, L>,
) -> Result<WorthQueryReplayNoiseContract, WorthQueryCertificationReplayAdmissionDenial> {
    match bound.definition().semantics().replay {
        WorthQueryOperationReplayContract::CertReplayable { .. } => {
            Ok(WorthQueryReplayNoiseContract::default())
        }
        WorthQueryOperationReplayContract::CertReplayableWithNoise { noise, .. } => Ok(noise),
        _ => Err(WorthQueryCertificationReplayAdmissionDenial::ReplayNotInstalled),
    }
}

fn intent_matches_original_trace(
    intent: &WorthQueryNormalizedWorkflowIntent,
    original_semantics: &WorthQueryWorkflowTraceSemantics,
    counters: &mut WorthQueryCertificationReplayCounters,
) -> bool {
    let original_stage_index = original_semantics
        .stages()
        .iter()
        .map(|stage| (stage.stage_identity(), stage))
        .collect::<std::collections::BTreeMap<_, _>>();
    counters.original_stage_index_entries = original_stage_index.len();
    if intent.stages().len() != original_semantics.stages().len() {
        return false;
    }
    intent.stages().iter().all(|intent_stage| {
        counters.intent_stage_checks += 1;
        original_stage_index
            .get(intent_stage.stage_identity())
            .is_some_and(|original_stage| {
                intent_stage
                    .input()
                    .semantically_matches(original_stage.input())
            })
    })
}
