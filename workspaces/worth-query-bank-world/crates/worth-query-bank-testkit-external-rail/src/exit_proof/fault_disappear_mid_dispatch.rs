//! Fault 5: disappear mid-dispatch -- the rail closes the connection before
//! any ledger record exists, distinguishing it from a lost response after a
//! real commit.

pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::{dispatch, inquire_status, LedgerStatus, RailExchangeOutcome};

pub use crate::facade::support::{attempt_for, correlation_for, spawn_rail, FRAME_TIMEOUT};
