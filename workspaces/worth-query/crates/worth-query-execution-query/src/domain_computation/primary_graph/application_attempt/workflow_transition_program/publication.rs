pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::publication::*;
#[path = "publication/assessment_evidence.rs"]
pub mod assessment_evidence;
pub use assessment_evidence::PerformedWorkflowAssessmentEvidence;
#[path = "publication/performed.rs"]
pub mod performed;
pub use performed::{PerformedWorkflowApproval, PerformedWorkflowTransition};
#[path = "publication/commit.rs"]
mod commit;
#[path = "publication/operation.rs"]
mod operation;
#[path = "publication/transition_replay.rs"]
pub mod transition_replay;

pub mod body;
pub use body::{WorkflowApprovalBodyDenial, WorkflowApprovalBodyPreparation};
pub use commit::project;
pub use commit::transition_entity_in_receipt;

/// What advancing a workflow instance produced. The `Awaiting*` variants name what the
/// instance needs next; `Completed` is a landed transition. The remaining variants say why
/// nothing landed, or why a landed commit could not be projected.
#[derive(Debug)]
pub enum WorkflowProgressOutcome {
    RecoveredCommitted(
        crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt,
    ),
    AwaitingProposal(RequiredWorkflowProposal),
    AwaitingActor(RequiredWorkflowActor),
    AwaitingAssessment(RequiredWorkflowAssessment),
    AwaitingCondition(RequiredWorkflowCondition),
    AwaitingOperation(RequiredWorkflowOperation),
    AwaitingEvidence(RequiredWorkflowEvidence),
    AwaitingApproval(RequiredWorkflowApproval),
    Completed(PerformedWorkflowTransition),
    /// The commit did not land. Fresh completion retains its actual performed
    /// transition; a later keyed call exposes only `RecoveredCommitted` data.
    Application(crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted),
    ProjectionDenied(super::super::WorthQueryApplicationCommitReceipt),
    /// A landed receipt is retained when its required projection read stops.
    ProjectionReadStopped {
        receipt: super::super::WorthQueryApplicationCommitReceipt,
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    },
    PreparationDenied(super::super::WorthQueryApplicationAttemptDenial),
    AuthenticationDenied(
        worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial,
    ),
    IdempotencyDenied(
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
}
