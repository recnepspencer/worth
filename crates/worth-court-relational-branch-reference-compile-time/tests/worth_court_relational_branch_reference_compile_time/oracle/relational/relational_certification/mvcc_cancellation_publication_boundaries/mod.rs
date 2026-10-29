#[allow(unused_imports)]
pub(crate) use {std::sync::Arc, std::sync::Barrier, worth_relational::facade::history::BranchId, worth_relational::facade::mvcc::RelationalCancellationSource, worth_relational::facade::mvcc::RelationalInterruptionBoundary, worth_relational::facade::mvcc::RelationalOperationControl, worth_relational::facade::mvcc::RelationalOperationInterruption, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::mvcc::RelationalTransactionIntent, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world};

#[test]
fn cancellation_immediately_before_the_supply_chain_critical_section_moves_nothing() {
    let host_request = std::sync::Arc::new(
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(),
    );
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(request_execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let before = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let control = worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted().with_injected_interruption_for_test(
        worth_relational::facade::mvcc::RelationalInterruptionBoundary::BeforeCriticalSection,
        worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled,
        std::num::NonZeroUsize::MIN,
    );
    let transaction = world
        .runtime
        .begin_branch_transaction_with_control(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(request_execution, transaction)
        .unwrap();

    let event = match world
        .runtime
        .publication_port()
        .compare_and_publish(request_execution, candidate)
    {
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Interrupted(event) => event,
        outcome => panic!("pre-critical cancellation must win: {outcome:?}"),
    };
    assert_eq!(
        event.boundary(),
        worth_relational::facade::mvcc::RelationalInterruptionBoundary::BeforeCriticalSection
    );
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before
    );
}

#[test]
fn cancellation_inside_the_supply_chain_critical_section_is_deferred() {
    let host_request = std::sync::Arc::new(
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(),
    );
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(request_execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let identity = world.runtime.main_branch_identity();
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let before = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let source = worth_relational::facade::mvcc::RelationalCancellationSource::new();
    let reached = std::sync::Arc::new(std::sync::Barrier::new(2));
    let release = std::sync::Arc::new(std::sync::Barrier::new(2));
    let control = worth_relational::facade::mvcc::RelationalOperationControl::from(source.token())
        .with_critical_section_pause_for_test(std::sync::Arc::clone(&reached), std::sync::Arc::clone(&release));
    let transaction = world
        .runtime
        .begin_branch_transaction_with_control(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(request_execution, transaction)
        .unwrap();
    let port = world.runtime.publication_port();
    let publisher = std::thread::spawn({
        let publication_request = std::sync::Arc::clone(&host_request);
        move || {
            let request_execution =
                &worth_execution::ExecutionRequest::serial(&publication_request);
            port.compare_and_publish(request_execution, candidate)
        }
    });

    reached.wait();
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before,
        "the certification pause is inside coordination but before movement"
    );
    source.cancel();
    release.wait();

    let performed = match publisher.join().unwrap() {
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("inside-critical cancellation must perform: {outcome:?}"),
    };
    assert_ne!(
        world
            .runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before
    );
    let interruption = performed
        .late_interruption()
        .expect("deferred cancellation remains attached to performed evidence");
    assert_eq!(
        interruption.boundary(),
        worth_relational::facade::mvcc::RelationalInterruptionBoundary::AfterLinearization
    );
    let committed = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}
