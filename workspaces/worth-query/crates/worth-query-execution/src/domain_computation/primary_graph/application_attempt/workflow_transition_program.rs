use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::effect_program::{admit_platform_effects, PlatformEffectDemand};
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::workflow::{
    definition::{reconstruct_compiled_definition, WorkflowDefinitionCompilationPosture},
    instance::{
        admit_workflow_transition, select_assessment_collection, select_current_transition,
        select_navigation_back_transition, select_settled_replay_transition,
        select_terminal_transition, visit_terminal_transition_facts,
        visit_workflow_transition_facts, SelectedWorkflowTransitionKind, WorkflowInstanceState,
    },
};

mod actor_context;
mod actor_selection;
pub use actor_context::WorthQueryWorkflowActorContext;
pub(in crate::domain_computation) use actor_selection::WorkflowRecordedActorObservation;
mod advance;
mod approval;
mod approval_decision;
mod assessment;
mod assessment_applicability;
mod assessment_coverage;
mod assessment_materialization;
mod await_inbound;
mod condition;
mod condition_materialization;
mod evidence_join;
mod local_actions;
mod navigation;
mod operation;
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

#[derive(Clone, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorkflowTransitionRequestKind {
    Advance,
    NavigateBack,
    LocalAction {
        kind: WorkflowLocalActionKind,
        requested:
            crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowOperation,
        input_identity: [u8; 32],
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
