use std::sync::{mpsc, Arc};
use std::time::Duration;

use super::fork_creation::setup_with_relational_source;
use super::*;

fn intent(fork: bool) -> ProductBranchCreationIntent {
    if fork {
        relational_fork_intent("child", "component-child")
    } else {
        reuse_intent("child")
    }
}

#[test]
fn pre_cancelled_creation_denies_both_routes_and_releases_capacity() {
    for fork in [false, true] {
        let (_fixture, owner, root) = setup_with_relational_source(3);
        let cancellation = RuntimeWorldCancellationSource::new();
        cancellation.cancel();
        let before = owner.state.retention.active_component_obligation_count();
        let result = owner.create_product_branch(RuntimeWorldBranchCreationRequest::new(
            root.clone(),
            intent(fork),
            &cancellation.token(),
        ));
        assert!(matches!(
            result,
            Err(RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect)
        ));
        assert_eq!(owner.state.branches.branch_count(), 1);
        assert_eq!(owner.state.branches.reserved_branch_count(), 0);
        assert_eq!(
            owner.state.retention.active_component_obligation_count(),
            before
        );
        assert_eq!(owner.recovery_record_count(), 0);
        // The same destination is usable when the caller has not cancelled.
        let healthy = owner
            .create_product_branch(RuntimeWorldBranchCreationRequest::new(
                root,
                intent(fork),
                &RuntimeWorldCancellationSource::new().token(),
            ))
            .unwrap();
        assert!(matches!(
            healthy,
            RuntimeWorldBranchCreationOutcome::Performed(_)
        ));
        assert_eq!(owner.state.operation.active(), 0);
    }
}

#[test]
fn cancellation_at_install_releases_reuse_but_retains_a_performed_fork() {
    for fork in [false, true] {
        let (_fixture, owner, root) = setup_with_relational_source(3);
        let owner = Arc::new(owner);
        let (reached, receiver) = mpsc::sync_channel(1);
        let gate = owner.rehearse_source_guarded_install(reached);
        let cancellation = RuntimeWorldCancellationSource::new();
        let token = cancellation.token();
        let worker_owner = Arc::clone(&owner);
        let worker = std::thread::spawn(move || {
            worker_owner.create_product_branch(RuntimeWorldBranchCreationRequest::new(
                root,
                intent(fork),
                &token,
            ))
        });
        // The gate's Drop releases the worker even if the assertion fails.
        receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("creation reaches installation");
        cancellation.cancel();
        drop(gate);
        let outcome = worker.join().expect("creation does not panic");
        if fork {
            let retained = match outcome {
                Ok(RuntimeWorldBranchCreationOutcome::ProductUnpublished(retained)) => retained,
                other => panic!("the real fork must survive cancellation: {other:?}"),
            };
            assert_eq!(retained.owner_effect_count(), 1);
            assert_eq!(
                retained.cause(),
                crate::recovery::ProductUnpublishedCause::CancellationAfterEffect
            );
            let handle = retained.recovery_handle();
            drop(retained);
            assert!(owner.cleanup_recovery_handle(&handle).is_some());
        } else {
            assert!(matches!(
                outcome,
                Err(RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect)
            ));
        }
        assert_eq!(owner.state.branches.branch_count(), 1);
        assert_eq!(owner.state.branches.reserved_branch_count(), 0);
        assert_eq!(owner.state.operation.active(), 0);
        assert_eq!(owner.recovery_record_count(), 0);
    }
}

#[test]
fn cancellation_at_signal_owner_cutoff_retains_only_the_real_relational_fork() {
    use super::fork_creation::{fork_intent, relational_fork, signal_fork};
    let (_fixture, owner, root) = setup_with_relational_source(3);
    let owner = Arc::new(owner);
    let (reached, receiver) = mpsc::sync_channel(1);
    let gate = owner.rehearse_signal_fork_cutoff(reached);
    let cancellation = RuntimeWorldCancellationSource::new();
    let token = cancellation.token();
    let worker_owner = Arc::clone(&owner);
    let source = root.clone();
    let worker = std::thread::spawn(move || {
        worker_owner.create_product_branch(RuntimeWorldBranchCreationRequest::new(
            source,
            fork_intent(
                "child",
                relational_fork("cancel-rel"),
                signal_fork("cancel-signal"),
            ),
            &token,
        ))
    });
    receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("real Signal fork reaches its cutoff");
    cancellation.cancel();
    drop(gate);
    let result = worker.join().unwrap().unwrap();
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(retained) = result else {
        panic!("Relational has forked but Signal must refuse")
    };
    assert_eq!(retained.owner_effect_count(), 1);
    assert_eq!(
        retained.cause(),
        crate::recovery::ProductUnpublishedCause::CancellationAfterEffect
    );
    assert_eq!(
        owner
            .observe_product_branch(root.branch_identity())
            .unwrap(),
        root
    );
    let handle = retained.recovery_handle();
    drop(retained);
    let report = owner.cleanup_recovery_handle(&handle).unwrap();
    drop(report);
    // A fresh fork to the same Signal destination proves the cancelled owner
    // call neither installed it nor leaked its reservation.
    let result = owner
        .create_product_branch(RuntimeWorldBranchCreationRequest::new(
            root,
            fork_intent(
                "signal-child",
                RelationalBranchCreationPlan::ReuseExact,
                signal_fork("cancel-signal"),
            ),
            &RuntimeWorldCancellationSource::new().token(),
        ))
        .unwrap();
    assert!(matches!(
        result,
        RuntimeWorldBranchCreationOutcome::Performed(_)
    ));
    assert_eq!(owner.recovery_record_count(), 0);
    assert_eq!(owner.state.operation.active(), 0);
}
