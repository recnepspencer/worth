use super::*;

#[test]
fn mixed_route_batch_completes_each_settlement_on_its_admitted_signal_route() {
    let root = tempdir().unwrap();
    let (profile, read_request, write_request, _) = family_locality_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let read = ready_read_work(&serving, read_request);
    let read_demand = worth_store::physical_runtime::PhysicalSchedulerDemand::foreground(
        read,
        worth_store_testkit_store::physical_work::scheduler_reservation::reserved_buffered_file_read(&serving),
        None,
    )
    .unwrap();
    let read_work = read_demand.queue_work();
    let backend_requirement = read_work.backend_requirement();
    let requested_budget = read_work.requested_budget();
    let read_backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let read_demand = secure_demand(read_demand, &read_backend);
    let read = serving
        .admit_physical_scheduler_demand(
            read_demand,
            &read_backend,
            policy_receipt_for(
                requested_budget,
                0,
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeRead,
            ),
        )
        .unwrap();
    let write = admitted_write(&serving, write_request);
    let identities = [read.intent().identity(), write.intent().identity()];

    let batch = serving.execute_physical_work_batch(
        vec![
            worth_store::physical_runtime::PhysicalExecutorCommand::read(read).unwrap(),
            worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
                write,
                b"routes!!".as_slice(),
            )
            .unwrap(),
        ]
        .into_boxed_slice(),
    );

    assert!(batch.denied_before_effect().is_empty());
    assert_eq!(batch.executions().len(), 2);
    assert_eq!(
        batch
            .executions()
            .iter()
            .map(|execution| execution.settled().intent().identity())
            .collect::<Vec<_>>(),
        identities
    );
    assert!(batch.executions().iter().all(|execution| execution.signal()
        == worth_store::physical_runtime::PhysicalSignalSettlementOutcome::Committed));
    let closed = serving.close_plan().execute();
    assert!(matches!(
        closed,
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
    let signal = closed.shutdown().signal_summary().unwrap();
    assert_eq!(signal.active_in_flight_node_count(), 0);
}
