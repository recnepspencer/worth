//! The relational_contracts integration suite: one test binary for the crate's
//! scenario files, which stay in tests/ with their own module trees.
//! Support modules several scenarios share are compiled once here.

#[path = "branch_reference_compile_time.rs"]
mod branch_reference_compile_time;
#[path = "branch_reference_contract.rs"]
mod branch_reference_contract;
#[path = "phase_boundaries_compile_fail.rs"]
mod phase_boundaries_compile_fail;
#[path = "relational_certification.rs"]
mod relational_certification;
#[allow(dead_code, unused_imports)]
#[path = "relational_certification/test_execution_policy.rs"]
pub mod shared_relational_certification_test_execution_policy;
#[path = "ui.rs"]
mod ui;
