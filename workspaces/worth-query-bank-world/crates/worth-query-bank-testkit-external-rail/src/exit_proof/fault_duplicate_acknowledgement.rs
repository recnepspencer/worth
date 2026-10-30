//! Fault 4: duplicate acknowledgement -- the rail sends the acknowledgement
//! twice, and neither one may be mistaken for completion.

pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::{dispatch, inquire_status, LedgerStatus, RailExchangeOutcome};

pub use crate::facade::support::{attempt_for, correlation_for, spawn_rail, FRAME_TIMEOUT};
