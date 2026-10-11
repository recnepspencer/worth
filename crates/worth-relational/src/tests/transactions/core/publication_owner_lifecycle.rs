use crate::tests::support::*;

#[test]
fn publication_port_denies_candidate_after_owner_drop() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "owner-drop-anchor");
    let basis =
        crate::tests::support::test_owner_main_basis(&runtime).expect("main basis is admitted");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .expect("transaction binds while its owner is live");
    transaction
        .push_batch(
            batch_create("owner-drop-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("candidate prepares while its owner is live");
    let publication_cell = candidate.publication_cell_for_test();
    let expected_root = candidate.expected_root_for_test();
    let branch_state_before = publication_cell
        .enter_state()
        .snapshot_cell()
        .evidence_state();
    let contacts_before = publication_cell.coordination().contact_count();
    let port = runtime.publication_port();
    let runtime_instance_id = runtime.runtime_instance_id();

    drop(runtime);

    match port.compare_and_publish(candidate) {
        crate::mvcc::RelationalPublicationOutcome::Denied(
            crate::mvcc::RelationalPublicationDenial::OwnerUnavailable {
                runtime_instance_id: denied_runtime_instance_id,
            },
        ) => assert_eq!(denied_runtime_instance_id, runtime_instance_id),
        outcome => panic!("a closed owner must deny without movement: {outcome:?}"),
    }
    let branch_state_after = publication_cell
        .enter_state()
        .snapshot_cell()
        .evidence_state();
    assert_eq!(branch_state_after, branch_state_before);
    assert!(publication_cell.currently_selects_root(&expected_root));
    assert_eq!(
        publication_cell.coordination().contact_count(),
        contacts_before,
        "owner denial occurs before branch-local linearization"
    );
}

#[test]
fn runtime_drop_waits_for_admitted_publication_to_leave_linearization() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "in-flight-drop-anchor");
    let basis =
        crate::tests::support::test_owner_main_basis(&runtime).expect("main basis is admitted");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .expect("transaction binds while its owner is live");
    transaction
        .push_batch(
            batch_create("in-flight-drop-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("candidate prepares while its owner is live");
    let publication_cell = candidate.publication_cell_for_test();
    let branch_gate = std::sync::Arc::clone(publication_cell.coordination());
    let held_branch_gate = branch_gate.enter();
    let port = runtime.publication_port();
    let shutdown_observer = runtime.owner_binding();

    let publisher = std::thread::spawn(move || port.compare_and_publish(candidate));
    let publication_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while branch_gate.wait_count() == 0 && std::time::Instant::now() < publication_deadline {
        std::thread::yield_now();
    }
    assert_eq!(
        branch_gate.wait_count(),
        1,
        "publisher reaches branch linearization after lifecycle admission"
    );

    let (drop_finished_sender, drop_finished_receiver) = std::sync::mpsc::sync_channel(1);
    let runtime_drop = std::thread::spawn(move || {
        drop(runtime);
        drop_finished_sender
            .send(())
            .expect("drop completion receiver lives");
    });
    let shutdown_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while shutdown_observer.admit().is_some() && std::time::Instant::now() < shutdown_deadline {
        std::thread::yield_now();
    }
    assert!(
        shutdown_observer.admit().is_none(),
        "runtime drop closes new publication admission"
    );
    assert_eq!(
        drop_finished_receiver.recv_timeout(std::time::Duration::from_millis(100)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout),
        "runtime drop waits for the admitted publisher while its branch gate remains held"
    );

    drop(held_branch_gate);
    let outcome = publisher
        .join()
        .unwrap_or_else(|worker_panic| std::panic::resume_unwind(worker_panic));
    assert!(
        matches!(
            outcome,
            crate::mvcc::RelationalPublicationOutcome::Performed(_)
        ),
        "already-admitted publication completes before owner shutdown: {outcome:?}"
    );
    join_completed_worker(
        drop_finished_receiver.recv_timeout(std::time::Duration::from_secs(1)),
        runtime_drop,
        std::time::Duration::from_secs(1),
        "runtime drop completes after publication leaves admission",
    );
}
