use worth_query_installation::facade::ApplicationSchema;

use super::{PreparedWorkflowAdvance, WorkflowProgressOutcome};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
    WorthQueryApplicationIdempotencyResolutionDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

#[doc(hidden)]
pub struct PreparedWorkflowTransitionReplay {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) identity:
        String,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) identity_bytes:
        [u8; 32],
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) node_path:
        String,
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowAdvance<Schema, Operation, Input, Scope> {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn with_replays(
        self,
        replays: Box<[PreparedWorkflowTransitionReplay]>,
    ) -> Self {
        match self {
            Self::Transition {
                program,
                program_revision,
                transition_identity,
                transition_identity_bytes,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                node_path,
                assessment,
                approval,
                approval_identity,
                ..
            } => Self::Transition {
                program,
                program_revision,
                transition_identity,
                transition_identity_bytes,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                node_path,
                assessment,
                approval,
                approval_identity,
                replays,
            },
            Self::AwaitingAssessment(mut prepared) => {
                prepared.replays = replays;
                Self::AwaitingAssessment(prepared)
            }
            Self::AwaitingEvidence {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                required,
                ..
            } => Self::AwaitingEvidence {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                required,
                replays,
            },
            Self::AwaitingApproval {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                required,
                ..
            } => Self::AwaitingApproval {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                required,
                replays,
            },
            Self::ReplayOnly {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                approval,
                approval_identity,
                denial,
                ..
            } => Self::ReplayOnly {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                approval,
                approval_identity,
                replays,
                denial,
            },
        }
    }

    pub(super) fn resolve_transition_replay(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<Option<WorkflowProgressOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let (admission, locator, approval_identity, approval, replays) = match self {
            Self::Transition {
                program,
                transition_identity_locator,
                approval,
                approval_identity,
                replays,
                ..
            } => (
                &program.read_set.admission,
                transition_identity_locator,
                *approval_identity,
                approval.as_ref(),
                replays.as_ref(),
            ),
            Self::AwaitingAssessment(prepared) => (
                &prepared.admitted.read_set().admission,
                &prepared.layout.transition.identity,
                None,
                None,
                prepared.replays.as_ref(),
            ),
            Self::AwaitingEvidence {
                read_set,
                transition_identity_locator,
                replays,
                ..
            }
            | Self::AwaitingApproval {
                read_set,
                transition_identity_locator,
                replays,
                ..
            }
            | Self::ReplayOnly {
                read_set,
                transition_identity_locator,
                replays,
                ..
            } => (
                &read_set.admission,
                transition_identity_locator,
                match self {
                    Self::ReplayOnly {
                        approval_identity, ..
                    } => *approval_identity,
                    _ => None,
                },
                match self {
                    Self::ReplayOnly { approval, .. } => approval.as_ref(),
                    _ => None,
                },
                replays.as_ref(),
            ),
        };
        if replays.is_empty() {
            return Ok(None);
        }
        let mut drift = false;
        let bindings = replays.iter().map(|replay| {
            approval_identity.map_or_else(
                || idempotency.bind_workflow_transition(&replay.identity_bytes),
                |approval| {
                    idempotency
                        .bind_workflow_transition(&replay.identity_bytes)
                        .bind_workflow_approval(&approval)
                },
            )
        });
        let resolutions =
            runtime.resolve_admitted_application_idempotencies(admission, bindings)?;
        for (replay, resolution) in replays.iter().zip(resolutions) {
            match resolution.into_resolution() {
                WorthQueryApplicationIdempotencyResolution::Unseen => {}
                WorthQueryApplicationIdempotencyResolution::IntentDrift => drift = true,
                WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => {
                    return Ok(Some(runtime.primary_provider.graph.with_runtime(
                        |relational| {
                            super::project(
                                relational,
                                receipt,
                                replay.identity.clone(),
                                locator.clone(),
                                replay.node_path.clone(),
                                None,
                                approval.cloned(),
                                true,
                            )
                        },
                    )))
                }
            }
        }
        Ok(drift.then(|| {
            WorkflowProgressOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
            ))
        }))
    }
}
