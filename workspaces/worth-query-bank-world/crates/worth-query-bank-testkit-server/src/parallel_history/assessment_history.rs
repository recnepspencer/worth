//! Real retained settlement checks typed phase outcomes and the independent journal.
//! The focused assessment protocol model is unit-tested separately; it does not
//! judges each actual advancement and its owner-applied order.
#![allow(
    clippy::result_large_err,
    reason = "Assessment history exercises complete native workflow refusals without narrowing their evidence"
)]

pub use super::journal_model;
pub use crate::facade::approved_payment_workflow::authentication;
pub use crate::facade::fixture;
pub use bank_domain::proposals::BankIdempotencyKey;
pub use bank_domain::schema::ApprovePayment;
pub use bank_server::{BankApprovedPaymentAssessmentProgress, BankPaymentAssessmentSettlement};
pub use std::num::NonZeroUsize;
pub use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
};
pub fn key(label: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(label).unwrap()
}
