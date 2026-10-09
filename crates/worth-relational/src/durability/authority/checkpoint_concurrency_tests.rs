use std::sync::mpsc::sync_channel;
use std::time::Duration;

use crate::facade::history::BranchId;
use crate::tests::support::*;

#[test]
fn checkpoint_reconstruction_does_not_exclude_publication_after_capture() {
    let runtime = runtime_with_test_schema();
    let baseline = create_entity_outcome(&runtime, "checkpoint-concurrency-anchor");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("publication-during-checkpoint-reconstruction"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("publication candidate prepares");
    let port = runtime.publication_port();
    let (captured_tx, captured_rx) = sync_channel(0);
    let (resume_tx, resume_rx) = sync_channel(0);

    let checkpoint_thread = std::thread::spawn(move || {
        let captured = runtime
            .durability_authority()
            .capture_checkpoint_basis()
            .expect("checkpoint captures one immutable recovery basis");
        captured_tx
            .send(())
            .expect("test observes completed checkpoint capture");
        resume_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("test releases checkpoint reconstruction");
        let checkpoint = runtime
            .durability_authority()
            .finalize_captured_checkpoint(captured);
        (runtime, checkpoint)
    });

    captured_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("checkpoint reaches post-capture reconstruction boundary");
    let (publication_tx, publication_rx) = sync_channel(1);
    let publication_thread = std::thread::spawn(move || {
        publication_tx
            .send(port.compare_and_publish(candidate))
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
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = outcome else {
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
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    assert_eq!(plan.tail_log.len(), 1);
    assert_eq!(plan.tail_log[0].position(), performed_position);
    assert_eq!(
        plan.tail_log[0].envelope().commit.commit_id,
        performed_commit_id
    );
    let mut recovered = runtime_with_test_schema();
    let outcome = recovered
        .durability_recovery()
        .recover(plan)
        .expect("fresh recovery replays the concurrent publication as checkpoint tail");
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .expect("recovered main head includes concurrent publication")
            .commit_id,
        performed_commit_id
    );
}
