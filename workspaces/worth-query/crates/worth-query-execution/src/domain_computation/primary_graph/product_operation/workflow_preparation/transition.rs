//! The workflow transition program's publication adapter: advance, navigation,
//! assessment collection, approval, local actions and their commits and replays.
//! Each entry that holds a selection hands the attempt its `WorkflowSelectionBasis`.
use crate::domain_computation::primary_graph::application_attempt::{
    PreparedWorkflowAdvance, RequiredWorkflowApproval, RequiredWorkflowOperation,
    WorkflowLocalActionKind, WorkflowProgressOutcome, WorkflowTransitionPreparationDenial,
};
use crate::domain_computation::primary_graph::application_installation::WorthQueryWorkflowVocabulary;
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

mod actor_denial;
mod adapter_body;
mod adapter_commit;
mod adapter_owner;
mod adapter_preparation;
mod adapter_replay;
mod local_operation;

#[doc(hidden)]
pub struct WorthQueryWorkflowAdvanceAdapter;
