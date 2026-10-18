//! The signal_contracts integration suite: one test binary for the crate's
//! scenario files, which stay in tests/ with their own module trees.
//! Support modules several scenarios share are compiled once here.

#[path = "branch_basis_contract.rs"]
mod branch_basis_contract;
#[path = "branch_operation_contract.rs"]
mod branch_operation_contract;
#[path = "branch_reference_compile_time.rs"]
mod branch_reference_compile_time;
#[path = "branch_reference_contract.rs"]
mod branch_reference_contract;
#[path = "branch_retention_contract.rs"]
mod branch_retention_contract;
#[path = "branch_retention_lifecycle.rs"]
mod branch_retention_lifecycle;
#[path = "execution_request_denials.rs"]
mod execution_request_denials;
#[path = "milestone_12_compile_time.rs"]
mod milestone_12_compile_time;
#[path = "milestone_13_compile_time.rs"]
mod milestone_13_compile_time;
#[path = "milestone_17_compile_time.rs"]
mod milestone_17_compile_time;
#[path = "phase_2a_signal_boundaries.rs"]
mod phase_2a_signal_boundaries;
#[path = "phase_5_policy_boundaries.rs"]
mod phase_5_policy_boundaries;
#[path = "signal_owner_services.rs"]
mod signal_owner_services;
