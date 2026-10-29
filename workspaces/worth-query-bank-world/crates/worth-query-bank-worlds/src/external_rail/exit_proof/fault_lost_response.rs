//! Fault 1: commit then lose the response.
//!
//! The caller's connection is closed with zero bytes, yet the rail's own
//! ledger -- reachable only by asking it, not by inference -- proves the
//! effect really completed. This is the exact indeterminacy Gate 8.2 must
//! not resolve by guessing.

pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::{dispatch, inquire_status, LedgerStatus, RailExchangeOutcome};

pub use crate::facade::external_rail::support::{
    attempt_for, correlation_for, spawn_rail, FRAME_TIMEOUT,
};
