//! Fast integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "fast/branch_basis_contract.rs"]
pub(crate) mod branch_basis_contract;

#[path = "fast/branch_operation_contract.rs"]
pub(crate) mod branch_operation_contract;

#[path = "fast/branch_reference_contract.rs"]
pub(crate) mod branch_reference_contract;

#[path = "fast/branch_retention_contract.rs"]
pub(crate) mod branch_retention_contract;

#[path = "fast/branch_retention_lifecycle.rs"]
pub(crate) mod branch_retention_lifecycle;

#[path = "fast/phase_5_policy_boundaries.rs"]
pub(crate) mod phase_5_policy_boundaries;

#[path = "fast/signal_owner_services.rs"]
pub(crate) mod signal_owner_services;

#[path = "fast/request_differential.rs"]
// `request_differential` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod request_differential;

/// The documented owner workflow runs whole, exactly as the example binary.
#[test]
fn branch_bases_owner_workflow_runs_as_documented() {
    branch_bases_example::main();
}
