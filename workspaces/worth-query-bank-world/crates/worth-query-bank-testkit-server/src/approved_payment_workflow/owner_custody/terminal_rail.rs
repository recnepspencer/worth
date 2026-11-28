//! Fixture for recovery of a completed payment rail effect.
pub use super::super::{
    assertions::{key, require_completed},
    fixture,
    ready_payment::ReadyPaymentWorld,
};
pub use bank_domain::schema::PaymentStatus;
pub use bank_external_rail::test_control::FaultScript;
pub use std::time::Duration;
pub use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[path = "terminal_rail/balance.rs"]
pub mod support;
