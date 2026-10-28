//! Compile fail integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "compile_fail/branch_reference_compile_time.rs"]
pub(crate) mod branch_reference_compile_time;

#[path = "compile_fail/milestone_12_compile_time.rs"]
pub(crate) mod milestone_12_compile_time;

#[path = "compile_fail/milestone_13_compile_time.rs"]
pub(crate) mod milestone_13_compile_time;

#[path = "compile_fail/milestone_17_compile_time.rs"]
pub(crate) mod milestone_17_compile_time;

#[path = "compile_fail/phase_2a_signal_boundaries.rs"]
pub(crate) mod phase_2a_signal_boundaries;

#[path = "compile_fail/phase_5_policy_boundaries.rs"]
pub(crate) mod phase_5_policy_boundaries;

#[path = "compile_fail/signal_owner_services.rs"]
pub(crate) mod signal_owner_services;

#[path = "compile_fail/tests/mod.rs"]
mod tests;
