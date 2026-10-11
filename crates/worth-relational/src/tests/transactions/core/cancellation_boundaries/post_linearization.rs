use super::*;

#[test]
fn cancellation_after_linearization_returns_the_performed_commit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "late-cancellation-anchor");
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let source = RelationalCancellationSource::new();
    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let control = RelationalOperationControl::from(source.token())
        .with_post_linearization_pause(Arc::clone(&reached), Arc::clone(&release));
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    transaction
        .push_batch(
            batch_create("late-cancellation-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let before = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let port = runtime.publication_port();

    let publisher = std::thread::spawn(move || port.compare_and_publish(candidate));
    reached.wait();
    assert_ne!(
        runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before,
        "the owner reference has already crossed its linearization point"
    );
    source.cancel();
    release.wait();

    let performed = match publisher.join().unwrap() {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("late cancellation cannot erase performed movement: {outcome:?}"),
    };
    let interruption = performed
        .late_interruption()
        .expect("late cancellation remains attached to the performed result");
    assert_eq!(
        interruption.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(
        interruption.boundary(),
        RelationalInterruptionBoundary::AfterLinearization
    );
    let committed = runtime
        .settle_performed_publication(performed)
        .expect("performed publication remains settleable after late cancellation");
    release_test_commit_snapshot(&runtime, &committed);
}
