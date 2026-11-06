//! The workflow transition program's publication adapter: advance, navigation,
//! assessment collection, approval, local actions and their commits and replays.
//! Each entry that holds a selection hands the attempt its `WorkflowSelectionBasis`.
use crate::domain_computation::primary_graph::application_attempt::{
    PreparedWorkflowAdvance, RequiredWorkflowApproval, RequiredWorkflowOperation,
    WorkflowLocalActionKind, WorkflowProgressOutcome, WorkflowTransitionPreparationDenial,
};
use crate::domain_computation::primary_graph::application_installation::WorthQueryWorkflowVocabulary;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::approval::WorkflowApprovalDecision;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQuerySelectedProductOperation,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

#[doc(hidden)]
pub struct WorthQueryWorkflowAdvanceAdapter;
