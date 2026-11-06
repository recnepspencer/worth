//! The workflow instance program's publication adapters: start, migration, fork
//! continuation, cancellation, creator enrollment and succession. Each entry that holds a
//! selection hands the attempt its `WorkflowSelectionBasis`.
use crate::domain_computation::primary_graph::application_attempt::workflow::instance::WorkflowSuccession;
use crate::domain_computation::primary_graph::application_attempt::{
    PreparedWorkflowInstanceCancellation, PreparedWorkflowInstanceStart,
    WorkflowInstanceCancellationOutcome, WorkflowInstancePreparationDenial,
    WorkflowInstanceStartOutcome,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQuerySelectedProductOperation,
};
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

mod adapter;
mod birth;
mod succession;

/// Prepares and commits every instance lifecycle request: start, migration,
/// fork continuation and cancellation. Each is admitted through the
/// application's instance binding, the same intent a start uses, so one
/// capability governs an instance from its start to its end.
#[doc(hidden)]
pub struct WorthQueryWorkflowInstanceAdapter;

#[doc(hidden)]
pub struct WorthQueryWorkflowBirthAdapter;

#[doc(hidden)]
pub struct WorthQueryWorkflowSuccessionAdapter;
