use super::*;

impl<Schema, Operation, Input, Scope> PreparedWorkflowAdvance<Schema, Operation, Input, Scope> {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn with_replays(
        self,
        replays: PreparedWorkflowTransitionReplays,
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
                terminal,
                assessment,
                supporting_identity,
                operation_receipt_identity,
                progress_update,
                approval,
                approval_identity,
                mut approval_authentication,
                ..
            } => {
                if let Some(authentication) = &mut approval_authentication {
                    authentication.bind_replays(&replays);
                }
                Self::Transition {
                    program,
                    program_revision,
                    transition_identity,
                    transition_identity_bytes,
                    transition_identity_locator,
                    assessment_identity_locator,
                    instance,
                    node_path,
                    terminal,
                    assessment,
                    supporting_identity,
                    operation_receipt_identity,
                    progress_update,
                    approval,
                    approval_identity,
                    approval_authentication,
                    replays,
                }
            }
            Self::AwaitingAssessment(mut prepared) => {
                prepared.replays = replays;
                Self::AwaitingAssessment(prepared)
            }
            Self::AwaitingCondition(mut prepared) => {
                prepared.replays = replays;
                Self::AwaitingCondition(prepared)
            }
            Self::AwaitingOperation(mut prepared) => {
                prepared.replays = replays;
                Self::AwaitingOperation(prepared)
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
}
