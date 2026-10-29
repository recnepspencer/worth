//! The approved-payment workflow's limit is an expression over the payment's
//! current amount: a payment over it is rejected before any approver acts.

pub use super::*;
pub use fixture::ordinary_read_world_with_payment_amount;
pub use worth_query_host::facade::application_entry::WorthQueryOrdinaryWorkflowRunStop;
