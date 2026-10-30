//! Fault 3: complete after timeout -- the rail delays past the caller's
//! deadline, then genuinely completes afterward.

pub use std::time::Duration;

pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::{dispatch, inquire_status, LedgerStatus, RailExchangeOutcome};

pub use crate::facade::support::{attempt_for, correlation_for, spawn_rail};

pub const CALLER_DEADLINE: Duration = Duration::from_millis(300);
pub const RAIL_DELAY_MILLIS: u64 = 1_000;
