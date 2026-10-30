//! P2 removes ordinary payment approval while preserving the installed schema.

pub use bank_domain::schema::ApprovePayment;
pub use bank_server::{BankApplicationP2, BankProgramAdoptionPreparationDenial};
pub use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorthQueryApplicationRequestMutationDenial, WorthQueryBranchAdoptionPublicationOutcome,
};

pub use super::assertions::key;
pub use super::authentication::approval_configuration;
pub use super::fixture::{
    ordinary_read_world_with_approval_authentication, principal_id, APPROVER,
};
pub use super::journey::start_approved_payment_instance;
pub use super::support::request_scope;
