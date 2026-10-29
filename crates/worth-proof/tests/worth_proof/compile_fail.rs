//! Compile fail integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "compile_fail/assumption_freshness_and_downgrade_certification.rs"]
// The proof_contracts integration suite: one test binary for the crate's
// scenario files, which stay in tests/ with their own module trees.
// Support modules several scenarios share are compiled once here.
pub(crate) mod assumption_freshness_and_downgrade_certification;

#[path = "compile_fail/blob_release_proof_contract.rs"]
pub(crate) mod blob_release_proof_contract;

#[path = "compile_fail/compile_fail_harness_contracts.rs"]
pub(crate) mod compile_fail_harness_contracts;

#[path = "compile_fail/dx_entrypoints_compile_pass.rs"]
pub(crate) mod dx_entrypoints_compile_pass;

#[path = "compile_fail/dx_scoped_defaults_compile_fail.rs"]
pub(crate) mod dx_scoped_defaults_compile_fail;

#[path = "compile_fail/fixed_shape_boundaries_compile_fail.rs"]
pub(crate) mod fixed_shape_boundaries_compile_fail;

#[path = "compile_fail/fork_join_boundaries_compile_fail.rs"]
pub(crate) mod fork_join_boundaries_compile_fail;

#[path = "compile_fail/fork_join_compile_pass.rs"]
pub(crate) mod fork_join_compile_pass;

#[path = "compile_fail/lowering_and_execution_readiness_boundary_certification.rs"]
pub(crate) mod lowering_and_execution_readiness_boundary_certification;

#[path = "compile_fail/phase_boundaries_compile_fail.rs"]
pub(crate) mod phase_boundaries_compile_fail;

#[path = "compile_fail/pleasant_lane_dx_certification.rs"]
pub(crate) mod pleasant_lane_dx_certification;

#[path = "compile_fail/sealed_minting_and_witness_authority_certification.rs"]
pub(crate) mod sealed_minting_and_witness_authority_certification;

#[path = "compile_fail/static_fork_join_and_composition_family_certification.rs"]
pub(crate) mod static_fork_join_and_composition_family_certification;

#[path = "compile_fail/transition_outcome_algebra_certification.rs"]
pub(crate) mod transition_outcome_algebra_certification;

#[path = "compile_fail/upfront_contract_boundaries.rs"]
pub(crate) mod upfront_contract_boundaries;
