//! The bridge_contracts integration suite: one test binary for the crate's
//! scenario files, which stay in tests/ with their own module trees.
//! Support modules several scenarios share are compiled once here.

#[path = "execution_request_denials.rs"]
mod execution_request_denials;
#[path = "no_inc_files.rs"]
mod no_inc_files;
#[path = "phase_boundaries_bridge_truth_identity_compile_fail.rs"]
mod phase_boundaries_bridge_truth_identity_compile_fail;
#[path = "phase_boundaries_compile_fail.rs"]
mod phase_boundaries_compile_fail;
#[path = "subscription_replay_folklore_guard.rs"]
mod subscription_replay_folklore_guard;
