//! Fresh admission policy distinct from binding-axis drift (Gate 8.3 turn 3).
//!
//! `ForeignPrincipal` is proved per-axis in `worth-query-execution`
//! `binding_axis_tests` — the default notify-death fixture only authorizes one
//! specialist principal on recovery re-admission.

pub use bank_external_rail::test_control::FaultScript;
pub use bank_server::{BankAuthorizationDenialKind, BankEstateProgressionDenial};

pub use super::phase8_cross_gate::world::{
    cross_gate_world_with_clock_and_grant_validity, PATIENT,
};
pub use crate::facade::server::authorization_time::AuthorizationTimeController;
pub use crate::facade::server::support::request_scope;
