use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::WorthQueryWorkspace;
use worth_proof::TransitionOutcome;

use super::{
    WorthQueryCompletedWorkflowTrace, WorthQueryExecutableDomainOperation,
    WorthQueryNormalizedWorkflowIntent, WorthQueryWorkflowAdvanceDenial,
    WorthQueryWorkflowCompletionDenial, WorthQueryWorkflowOperation, WorthQueryWorkflowRunCounters,
    WorthQueryWorkflowStartDenial,
};
use crate::domain_installation::WorthQueryAdmittedWorkflowOperation;

#[derive(Debug)]
pub enum WorthQueryWorkflowReexecutionStop {
    IntentDoesNotMatchInstalledGraph,
    ResourceAdmission(super::WorthQueryExecutionResourceAdmissionDenial),
    Start(WorthQueryWorkflowStartDenial),
    Advance(WorthQueryWorkflowAdvanceDenial),
    ConditionalDeferred {
        stage_identity: String,
        executed_effects: Vec<super::WorthQueryWorkflowEffectEvidence>,
    },
    OperationConditionalDeferred {
        conditional: Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
        counters: WorthQueryWorkflowRunCounters,
    },
    Completion(WorthQueryWorkflowCompletionDenial),
}

impl WorthQueryWorkflowReexecutionStop {
    pub fn executed_effects(&self) -> &[super::WorthQueryWorkflowEffectEvidence] {
        match self {
            Self::Advance(denial) => denial.executed_effects(),
            Self::ConditionalDeferred {
                executed_effects, ..
            } => executed_effects,
            Self::Completion(denial) => denial.executed_effects(),
            Self::IntentDoesNotMatchInstalledGraph
            | Self::ResourceAdmission(_)
            | Self::Start(_)
            | Self::OperationConditionalDeferred { .. } => &[],
        }
    }
}

pub type WorthQueryWorkflowReexecutionOutcome<D, O, F, L> = TransitionOutcome<
    WorthQueryCompletedWorkflowTrace<D, O, F, L>,
    WorthQueryWorkflowReexecutionStop,
    WorthQueryWorkflowReexecutionStop,
    WorthQueryWorkflowReexecutionStop,
    WorthQueryWorkflowReexecutionStop,
    WorthQueryWorkflowReexecutionStop,
>;

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryAdmittedWorkflowOperation<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<D, F, Execution = WorthQueryWorkflowOperation>,
{
    /// Executes normalized installed intent through the ordinary workflow lane.
    /// No prior trace or certification authority is accepted by this surface.
    pub fn reexecute(
        self,
        intent: WorthQueryNormalizedWorkflowIntent,
        workspace: &mut WorthQueryWorkspace,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQueryWorkflowReexecutionOutcome<D, O, F, L> {
        let mut run = match self.start_workflow(workspace) {
            TransitionOutcome::Success(run) => run,
            TransitionOutcome::Denied(stop) => {
                return TransitionOutcome::Denied(WorthQueryWorkflowReexecutionStop::Start(stop))
            }
            TransitionOutcome::Stale(stop) => {
                return TransitionOutcome::Stale(WorthQueryWorkflowReexecutionStop::Start(stop))
            }
            TransitionOutcome::Deferred(deferred) => {
                return TransitionOutcome::Deferred(
                    WorthQueryWorkflowReexecutionStop::OperationConditionalDeferred {
                        conditional: deferred.conditional,
                        counters: deferred.counters,
                    },
                )
            }
            TransitionOutcome::RebindRequired(stop) => {
                return TransitionOutcome::RebindRequired(WorthQueryWorkflowReexecutionStop::Start(
                    stop,
                ))
            }
            TransitionOutcome::Failed(stop) => {
                return TransitionOutcome::Failed(WorthQueryWorkflowReexecutionStop::Start(stop))
            }
        };
        if intent.stages().len() != run.installed_graph().stages().len() {
            return TransitionOutcome::Denied(
                WorthQueryWorkflowReexecutionStop::IntentDoesNotMatchInstalledGraph,
            );
        }
        for stage in intent.stages() {
            let advanced = match stage.input() {
                super::WorthQueryWorkflowIntentValue::PredecessorArtifact { predecessor_stage } => {
                    run.advance_with_artifact(
                        stage.stage_identity(),
                        predecessor_stage,
                        workspace,
                        request,
                    )
                }
                super::WorthQueryWorkflowIntentValue::PredecessorArtifactLease {
                    predecessor_stage,
                    lease_role,
                } => run.advance_with_artifact_lease(
                    stage.stage_identity(),
                    predecessor_stage,
                    lease_role.clone(),
                    workspace,
                    request,
                ),
                input => run.advance(
                    stage.stage_identity(),
                    input
                        .runtime_value()
                        .expect("non-artifact intent has a primitive runtime value"),
                    workspace,
                    request,
                ),
            };
            run = match advanced {
                TransitionOutcome::Success(run) => run,
                TransitionOutcome::Denied(stop) => {
                    return TransitionOutcome::Denied(WorthQueryWorkflowReexecutionStop::Advance(
                        stop,
                    ))
                }
                TransitionOutcome::Deferred(deferred) => {
                    let executed_effects = deferred
                        .run
                        .receipts()
                        .iter()
                        .flat_map(|receipt| receipt.effect_evidence().iter().cloned())
                        .collect();
                    return TransitionOutcome::Deferred(
                        WorthQueryWorkflowReexecutionStop::ConditionalDeferred {
                            stage_identity: stage.stage_identity().to_owned(),
                            executed_effects,
                        },
                    );
                }
                TransitionOutcome::Stale(stop) => {
                    return TransitionOutcome::Stale(WorthQueryWorkflowReexecutionStop::Advance(
                        stop,
                    ))
                }
                TransitionOutcome::RebindRequired(stop) => {
                    return TransitionOutcome::RebindRequired(
                        WorthQueryWorkflowReexecutionStop::Advance(stop),
                    )
                }
                TransitionOutcome::Failed(stop) => {
                    return TransitionOutcome::Failed(WorthQueryWorkflowReexecutionStop::Advance(
                        stop,
                    ))
                }
            };
        }
        match run.complete() {
            TransitionOutcome::Success(trace) => TransitionOutcome::Success(trace),
            TransitionOutcome::Denied(stop) => {
                TransitionOutcome::Denied(WorthQueryWorkflowReexecutionStop::Completion(stop))
            }
            TransitionOutcome::Stale(stop) => {
                TransitionOutcome::Stale(WorthQueryWorkflowReexecutionStop::Completion(stop))
            }
            TransitionOutcome::Deferred(never) => match never {},
            TransitionOutcome::RebindRequired(stop) => TransitionOutcome::RebindRequired(
                WorthQueryWorkflowReexecutionStop::Completion(stop),
            ),
            TransitionOutcome::Failed(stop) => {
                TransitionOutcome::Failed(WorthQueryWorkflowReexecutionStop::Completion(stop))
            }
        }
    }
}
