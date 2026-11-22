pub(crate) mod internal_port;
pub mod operation;
pub use operation::{
    PreparedWorkflowOperation, RequiredWorkflowOperation, WorkflowOperationAuthority,
    WorkflowOperationAuthoritySlot,
};
pub mod transition_replay {
    pub mod descriptor_retention;
    pub use descriptor_retention::PreparedWorkflowTransitionReplays;
}
pub use authentication::PreparedWorkflowApprovalAuthentication;
pub use transition_replay::PreparedWorkflowTransitionReplays;
pub mod authentication;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

use super::super::{WorthQueryApplicationEffectProgram, WorthQueryApplicationUncommitted};

#[path = "publication/actor.rs"]
pub mod actor;
#[path = "publication/approval.rs"]
pub mod approval;
#[path = "publication/evidence_readiness.rs"]
pub mod evidence_readiness;
#[path = "publication/required_assessment.rs"]
pub mod required_assessment;
#[path = "publication/required_proposal.rs"]
pub mod required_proposal;
pub use actor::RequiredWorkflowActor;
pub use approval::{RequiredWorkflowApproval, WorkflowApprovalDecision};
pub use evidence_readiness::RequiredWorkflowEvidence;
pub use required_assessment::RequiredWorkflowAssessment;
pub use required_proposal::RequiredWorkflowProposal;

#[must_use = "prepared workflow advance owns a live application attempt"]
pub enum PreparedWorkflowAdvance<Schema, Operation, Input, Scope> {
    Transition {
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        program_revision: ApplicationProgramRevision,
        transition_identity: String,
        transition_identity_bytes: [u8; 32],
        transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
        assessment_identity_locator: worth_foundational::facade::AspectFieldLocator,
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        terminal: bool,
        assessment: Option<PreparedWorkflowAssessmentProjection>,
        supporting_identity: Option<[u8; 32]>,
        operation_receipt_identity: Option<[u8; 32]>,
        progress_update: Option<
            crate::domain_computation::primary_graph::workflow::instance::PreparedWorkflowProgressUpdate,
        >,
        approval: Option<PreparedWorkflowApprovalProjection>,
        approval_identity: Option<[u8; 32]>,
        approval_authentication: Option<PreparedWorkflowApprovalAuthentication<Schema>>,
        replays: PreparedWorkflowTransitionReplays,
    },
    AwaitingAssessment(PreparedWorkflowAssessment<Schema, Operation, Input, Scope>),
    AwaitingCondition(PreparedWorkflowCondition<Schema, Operation, Input, Scope>),
    AwaitingOperation(PreparedWorkflowOperation<Schema, Operation, Input, Scope>),
    AwaitingProposal {
        read_set: super::super::WorthQueryCompleteApplicationReadSet<
            Schema, Operation, Input, Scope, super::super::WorthQueryProjectedApplicationMutation,
        >,
        transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
        assessment_identity_locator: worth_foundational::facade::AspectFieldLocator,
        instance: worth_relational::facade::identity::EntityId,
        required: RequiredWorkflowProposal,
        replays: PreparedWorkflowTransitionReplays,
    },
    AwaitingEvidence {
        read_set: super::super::WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            super::super::WorthQueryProjectedApplicationMutation,
        >,
        transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
        assessment_identity_locator: worth_foundational::facade::AspectFieldLocator,
        instance: worth_relational::facade::identity::EntityId,
        required: RequiredWorkflowEvidence,
        replays: PreparedWorkflowTransitionReplays,
    },
    AwaitingApproval {
        read_set: super::super::WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            super::super::WorthQueryProjectedApplicationMutation,
        >,
        transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
        assessment_identity_locator: worth_foundational::facade::AspectFieldLocator,
        instance: worth_relational::facade::identity::EntityId,
        required: RequiredWorkflowApproval,
        replays: PreparedWorkflowTransitionReplays,
    },
    ReplayOnly {
        read_set: super::super::WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            super::super::WorthQueryProjectedApplicationMutation,
        >,
        transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
        assessment_identity_locator: worth_foundational::facade::AspectFieldLocator,
        instance: worth_relational::facade::identity::EntityId,
        approval: Option<PreparedWorkflowApprovalProjection>,
        approval_identity: Option<[u8; 32]>,
        approval_authentication: Option<PreparedWorkflowApprovalAuthentication<Schema>>,
        replays: PreparedWorkflowTransitionReplays,
        denial: super::super::WorthQueryApplicationAttemptDenial,
    },
}

#[derive(Clone)]
#[doc(hidden)]
pub struct PreparedWorkflowApprovalProjection {
    pub identity: String,
    pub identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub fields: Vec<(
        worth_foundational::facade::AspectFieldLocator,
        worth_foundational::facade::AspectValue,
    )>,
    pub required_fields: Vec<worth_foundational::facade::AspectFieldLocator>,
    pub decision: WorkflowApprovalDecision,
    pub proposal: worth_relational::facade::identity::EntityId,
    pub evidence: Box<[worth_relational::facade::identity::EntityId]>,
    pub approver: worth_relational::facade::identity::EntityId,
    pub purpose: String,
    pub expiry: u64,
    pub transition_relation: worth_relational::facade::identity::KindId,
    pub proposal_relation: worth_relational::facade::identity::KindId,
    pub evidence_relation: worth_relational::facade::identity::KindId,
}

#[doc(hidden)]
pub struct PreparedWorkflowAssessmentProjection {
    pub identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub identity: String,
    pub intent_identity: [u8; 32],
    pub producer: String,
    pub family: String,
    pub query: String,
    pub parameter_type: String,
    pub result_type: String,
    pub binding: String,
    pub subject: worth_relational::facade::identity::EntityId,
    pub proposal_identity: String,
    pub coverage_identity: String,
    pub source_identity: String,
    pub passing: bool,
    pub publication_identity: String,
    pub output_content_identity: String,
}

pub struct PreparedWorkflowAssessment<Schema, Operation, Input, Scope> {
    pub admitted:
        crate::domain_computation::primary_graph::workflow::instance::AdmittedWorkflowTransition<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    pub required: RequiredWorkflowAssessment,
    pub applicability_dependencies:
        Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
    pub layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    pub program_revision: ApplicationProgramRevision,
    pub replays: PreparedWorkflowTransitionReplays,
    /// Evidence bytes the lineage may still retain when this settles.
    pub evidence_allowance: u64,
}

#[path = "publication/condition.rs"]
pub mod condition;
pub use condition::{PreparedWorkflowCondition, RequiredWorkflowCondition};

impl<Schema, Operation, Input, Scope> PreparedWorkflowAssessment<Schema, Operation, Input, Scope> {
    pub const fn required(&self) -> &RequiredWorkflowAssessment {
        &self.required
    }

    pub fn into_required(self) -> RequiredWorkflowAssessment {
        self.required
    }
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowAdvance<Schema, Operation, Input, Scope> {
    pub fn with_replays(self, replays: PreparedWorkflowTransitionReplays) -> Self {
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
            Self::AwaitingProposal {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                required,
                ..
            } => Self::AwaitingProposal {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                required,
                replays,
            },
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
                approval_authentication,
                denial,
                ..
            } => Self::ReplayOnly {
                read_set,
                transition_identity_locator,
                assessment_identity_locator,
                instance,
                approval,
                approval_identity,
                approval_authentication,
                replays,
                denial,
            },
        }
    }
}
