//! `leased_preparation` keeps its own test process: it constructs process-wide
//! state that the shared suite binary cannot host twice.

#[path = "../leased_preparation.rs"]
mod leased_preparation;
#[allow(dead_code, unused_imports)]
#[path = "../relational_certification/test_execution_policy.rs"]
pub mod shared_relational_certification_test_execution_policy;
