//! An older capture's completion cannot replace the compacted transition basis.
//! Restored from bbbce86edd:crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs
//! (`completing_an_older_capture_cannot_replace_the_compacted_transition_basis`).
//! Fixture adaptations (C2 ruling 2026-11-21): the runtime owns a
//! `TestDurableMedium` attachment, so a covering checkpoint really compacts its
//! branch transitions (the original used the retired in-crate medium); the
//! private capture/finalize split is reached through the owner's test doors
//! `capture_checkpoint_basis_for_test` and `finalize_captured_checkpoint_for_test`;
//! the private `latest_checkpoint()` read is the public retained recovery plan,
//! which selects the same last retained checkpoint.
use super::*;
use worth_relational::facade::history::BranchId;

#[test]
fn completing_an_older_capture_cannot_replace_the_compacted_transition_basis() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "source");
    let old = runtime
        .durability_authority()
        .capture_checkpoint_basis_for_test()
        .unwrap();
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId("target".into()), source)
        .unwrap();
    let new = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    assert_eq!(new.coverage.up_to_branch_transition.0, 1);
    runtime
        .durability_authority()
        .finalize_captured_checkpoint_for_test(old, execution)
        .unwrap();
    let selected = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
        .checkpoint
        .unwrap();
    assert_eq!(selected.coverage.up_to_branch_transition.0, 1);
    assert!(runtime
        .durability()
        .branch_transition_artifacts(execution.allocation_policy())
        .next()
        .is_none());
}
