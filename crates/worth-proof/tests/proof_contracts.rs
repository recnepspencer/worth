//! The proof_contracts integration suite: one test binary for the crate's
//! scenario files, which stay in tests/ with their own module trees.
//! Support modules several scenarios share are compiled once here.

#[path = "assumption_freshness_and_downgrade_certification.rs"]
mod assumption_freshness_and_downgrade_certification;
#[path = "band_guard_cargo_contract.rs"]
mod band_guard_cargo_contract;
#[path = "blob_release_proof_contract.rs"]
mod blob_release_proof_contract;
#[path = "compile_fail_harness_contracts.rs"]
mod compile_fail_harness_contracts;
#[path = "core_artifact_and_proof_substrate_certification.rs"]
mod core_artifact_and_proof_substrate_certification;
#[path = "dx_entrypoint_certification.rs"]
mod dx_entrypoint_certification;
#[path = "dx_entrypoints_compile_pass.rs"]
mod dx_entrypoints_compile_pass;
#[path = "dx_scoped_defaults_compile_fail.rs"]
mod dx_scoped_defaults_compile_fail;
#[path = "fixed_shape_boundaries_compile_fail.rs"]
mod fixed_shape_boundaries_compile_fail;
#[path = "fork_join_boundaries_compile_fail.rs"]
mod fork_join_boundaries_compile_fail;
#[path = "fork_join_compile_pass.rs"]
mod fork_join_compile_pass;
#[path = "lowering_and_execution_readiness_boundary_certification.rs"]
mod lowering_and_execution_readiness_boundary_certification;
#[path = "phase_boundaries_compile_fail.rs"]
mod phase_boundaries_compile_fail;
#[path = "pleasant_lane_dx_certification.rs"]
mod pleasant_lane_dx_certification;
#[path = "sealed_minting_and_witness_authority_certification.rs"]
mod sealed_minting_and_witness_authority_certification;
#[allow(dead_code, unused_imports)]
#[path = "support/mod.rs"]
pub mod shared_support;
#[path = "static_fork_join_and_composition_family_certification.rs"]
mod static_fork_join_and_composition_family_certification;
#[path = "supported_public_workflows.rs"]
mod supported_public_workflows;
#[path = "transition_outcome_algebra_certification.rs"]
mod transition_outcome_algebra_certification;
#[path = "upfront_contract_boundaries.rs"]
mod upfront_contract_boundaries;
#[path = "upfront_contract_vocabulary_certification.rs"]
mod upfront_contract_vocabulary_certification;
