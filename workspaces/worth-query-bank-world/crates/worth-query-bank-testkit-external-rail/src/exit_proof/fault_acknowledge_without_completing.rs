//! Fault 2: acknowledge without completing.

pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::{
    dispatch, inquire_admission_count, inquire_completed_effect_count, inquire_completed_notice,
    inquire_notice, inquire_status, LedgerStatus, RailExchangeOutcome,
};

pub use crate::facade::support::{attempt_for, correlation_for, spawn_rail, FRAME_TIMEOUT};
