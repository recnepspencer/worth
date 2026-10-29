//! Bank payment workflow, separate rail consequence, and real callback HTTP.
//!
//! The approval principals enter through Bank's causal Query authentication
//! adapter. The callback crosses a separate OS process and the production HTTP
//! verifier; this court does not claim an OIDC payment entry route.

#![allow(
    clippy::result_large_err,
    reason = "Transport payment courts retain complete native workflow and cleanup refusal owners"
)]

pub use std::time::{Duration, Instant};
pub use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub use bank_domain::queries::payment;
pub use bank_domain::schema::{ApprovePayment, PaymentStatus};
pub use bank_external_rail::{inquire_completed_effect_count, inquire_completion_delivery_posture};
pub use bank_server::BankApprovedPaymentWorkflowError;
pub use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
pub use worth_query_host::facade::application_entry::{
    WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryOperationAuthorizationDenialKind,
};

pub use crate::facade::courtroom::support;

#[path = "payment/authority.rs"]
pub mod authority;
#[path = "payment/cancellation.rs"]
pub mod cancellation;
#[path = "payment/close.rs"]
pub mod close;
#[path = "payment/oracle.rs"]
pub mod oracle;
#[path = "payment/retirement.rs"]
pub mod retirement;
#[path = "payment/setup.rs"]
pub mod setup;

pub use crate::facade::server::approved_payment_workflow::approval;
pub use crate::facade::server::approved_payment_workflow::assertions;
pub use crate::facade::server::approved_payment_workflow::authentication;
pub use crate::facade::server::approved_payment_workflow::journey;
pub use crate::facade::server::approved_payment_workflow::rail_transport;
pub use crate::facade::server::fixture;

pub use super::rail_completion::proxy::{correlation_token, CallbackProxy};
pub use assertions::{key, require_completed};
pub use fixture::{principal_id, APPROVER};
pub use setup::PaymentCourt;

pub const TIMEOUT: Duration = Duration::from_secs(5);
