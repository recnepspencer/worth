//! What one cancellation reaches: only the copy of the instance on the
//! request's own branch, every effect a migrated instance carried from its
//! source, and nothing at all without the start capability.

pub use worth_query_host::facade::application_entry::{
    WorkflowInstancePreparationDenial, WorthQueryApplicationRequestMutationDenial,
    WorthQueryWorkflowInstancePreparationDenial,
};

pub use super::super::document_retention_model::workflow::{
    cancel_instance, continue_on_fork, migrate_instance, prepare_cancellation_on,
    WorkflowAdvanceInput, WorkflowAdvanceIntent, WorkflowApprovalIntent,
};
pub use super::fork_continuation::fork_of;
pub use super::instance_cancellation::{approve, cancellation_denial, cancelled};
pub use super::instance_migration::{perform_approved_effect, started, supersede};
pub use super::*;
