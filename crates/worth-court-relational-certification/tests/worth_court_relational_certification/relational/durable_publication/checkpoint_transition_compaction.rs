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
use std::{sync::mpsc::sync_channel, time::Duration};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::RelationalPublicationOutcome;
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    durable_publication::{batch_create, create_entity_outcome, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

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

#[test]
fn older_checkpoint_completion_keeps_the_newest_transition_basis() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = memory_runtime();
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
}

#[test]
fn checkpoint_reconstruction_does_not_exclude_publication_after_capture() {
    let host_request = std::sync::Arc::new(serial_request());
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    let baseline = create_entity_outcome(execution, &runtime, "checkpoint-concurrency-anchor");
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            batch_create("publication-during-checkpoint-reconstruction"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .expect("publication candidate prepares");
    let port = runtime.publication_port();
    let (captured_tx, captured_rx) = sync_channel(0);
    let (resume_tx, resume_rx) = sync_channel(0);

    let checkpoint_request = std::sync::Arc::clone(&host_request);
    let checkpoint_thread = std::thread::spawn(move || {
        let captured = runtime
            .durability_authority()
            .capture_checkpoint_basis_for_test()
            .expect("checkpoint captures one immutable recovery basis");
        captured_tx
            .send(())
            .expect("test observes completed checkpoint capture");
        resume_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("test releases checkpoint reconstruction");
        let checkpoint = runtime
            .durability_authority()
            .finalize_captured_checkpoint_for_test(
                captured,
                &worth_execution::ExecutionRequest::serial(&checkpoint_request),
            );
        (runtime, checkpoint)
    });

    captured_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("checkpoint reaches post-capture reconstruction boundary");
    let (publication_tx, publication_rx) = sync_channel(1);
    let publication_request = std::sync::Arc::clone(&host_request);
    let publication_thread = std::thread::spawn(move || {
        let execution = worth_execution::ExecutionRequest::serial(&publication_request);
        publication_tx
            .send(port.compare_and_publish(&execution, candidate))
            .expect("publication outcome receiver remains live");
    });
    let outcome = match publication_rx.recv_timeout(Duration::from_secs(1)) {
        Ok(outcome) => outcome,
        Err(error) => {
            let _ = resume_tx.send(());
            let _ = checkpoint_thread.join();
            let _ = publication_thread.join();
            panic!("publication remained blocked after checkpoint capture: {error}");
        }
    };
    publication_thread
        .join()
        .expect("bounded publication thread joins");
    let RelationalPublicationOutcome::Performed(performed) = outcome else {
        panic!("publication must complete while checkpoint reconstruction is paused");
    };
    let performed_commit_id = performed.canonical_commit().commit.commit_id;
    let performed_position = performed.patch_position();
    resume_tx
        .send(())
        .expect("checkpoint reconstruction resumes after publication");

    let (runtime, checkpoint) = checkpoint_thread.join().expect("checkpoint thread joins");
    let checkpoint = checkpoint.expect("the pre-publication checkpoint remains valid");
    assert_eq!(
        checkpoint
            .coverage
            .up_to_commit
            .as_ref()
            .map(|commit| commit.commit_id),
        Some(baseline.commit.commit_id)
    );
    runtime
        .settle_performed_publication(performed)
        .expect("publication settles after checkpoint completes");
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    assert_eq!(plan.tail_log_len(), 1);
    let mut recovered = memory_runtime();
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("fresh recovery replays the concurrent publication as checkpoint tail");
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    assert_eq!(
        recovered
            .history()
            .immutable_commit_receipt_at_patch_stream_position(performed_position)
            .expect("recovered tail is readmitted at its performed position")
            .commit_id,
        performed_commit_id
    );
    let (_, recovered_head) = recovered
        .observe_branch(&recovered.main_branch_identity())
        .unwrap();
    assert_eq!(
        recovered
            .history()
            .branch_head_for_observation(&recovered_head.observation())
            .unwrap()
            .unwrap()
            .commit_id,
        performed_commit_id,
    );
}
