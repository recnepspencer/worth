use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use {
    crate::domain_computation::primary_graph::application_attempt::workflow::definition::reconstruct_compiled_definition,
    crate::domain_computation::primary_graph::application_attempt::workflow::definition::WorkflowDefinitionCompilationPosture,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::admit_workflow_transition,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::select_assessment_collection,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::select_current_transition,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::select_navigation_back_transition,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::select_settled_replay_transition,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::select_terminal_transition,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::visit_terminal_transition_facts,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::visit_workflow_transition_facts,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::SelectedWorkflowTransitionKind,
    crate::domain_computation::primary_graph::application_attempt::workflow::instance::WorkflowInstanceState,
};

mod actor_context;
pub(crate) mod actor_selection;
pub use actor_context::WorthQueryWorkflowActorContext;
pub(in crate::domain_computation) use actor_selection::WorkflowRecordedActorObservation;
mod advance;
mod approval;
mod approval_decision;
pub(crate) mod assessment;
mod assessment_applicability;
mod assessment_coverage;
mod assessment_materialization;
mod await_inbound;
mod condition;
mod condition_materialization;
mod evidence_join;
mod local_actions;
mod navigation;
pub(crate) mod operation;
pub(in crate::domain_computation::primary_graph::application_attempt) use operation::LocalActionSettlementProof;
pub(in crate::domain_computation::primary_graph) use operation::ValidatedWorkflowOperationReceipt;
pub(super) use operation::{operation_receipt_requires_recovery, receipt_identity_from_outcome};
mod preparation;
mod publication;
pub(super) use publication::transition_entity_in_receipt;
mod terminal;

pub use preparation::{
    WorkflowLocalActionKind, WorkflowTransitionBindingDenial, WorkflowTransitionPreparationDenial,
    WorthQueryWorkflowAdvanceAdapter,
};
pub use publication::{
    PerformedWorkflowApproval, PerformedWorkflowAssessmentEvidence, PerformedWorkflowTransition,
    PreparedWorkflowAdvance, PreparedWorkflowAssessment, PreparedWorkflowCondition,
    PreparedWorkflowOperation, RequiredWorkflowActor, RequiredWorkflowApproval,
    RequiredWorkflowAssessment, RequiredWorkflowCondition, RequiredWorkflowEvidence,
    RequiredWorkflowOperation, RequiredWorkflowProposal, WorkflowApprovalBodyDenial,
    WorkflowApprovalBodyPreparation, WorkflowApprovalDecision, WorkflowOperationAuthority,
    WorkflowOperationAuthoritySlot, WorkflowProgressOutcome,
};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) enum WorkflowTransitionRequestKind {
    Advance,
    NavigateBack,
    LocalAction {
        kind: WorkflowLocalActionKind,
        requested:
            crate::domain_computation::primary_graph::application_attempt::workflow::instance::SelectedWorkflowOperation,
        input_identity: [u8; 32],
        completion: Option<local_actions::VerifiedLocalActionCompletion>,
    },
    CollectAssessment {
        node_path: String,
    },
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
