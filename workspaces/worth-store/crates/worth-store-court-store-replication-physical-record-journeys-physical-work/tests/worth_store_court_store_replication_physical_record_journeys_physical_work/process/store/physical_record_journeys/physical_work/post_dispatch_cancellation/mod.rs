use {
    worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write,
    worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_one_write_pause_and_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

#[test]
fn cancellation_after_backend_dispatch_retains_terminal_settlement_obligation() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let (serving, gate) = serving_from_open_with_one_write_pause_and_profile(root.path(), profile);
    let admitted = admitted_write(&serving, request);
    let identity = admitted.intent().identity();
    let consumer = admitted.consumer_handle();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"retained".as_slice(),
    )
    .unwrap();
    let before = serving.media_counters();

    let settled = std::thread::scope(|scope| {
        let execution = scope.spawn(|| serving.execute_physical_work(command));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while gate.reached_context().is_none() && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        if gate.reached_context().is_none() {
            gate.release();
            let _ = execution.join();
            panic!("the real write did not reach the paused backend dispatch");
        }
        let cancellation = serving.cancel_physical_work(consumer).unwrap();
        assert_eq!(
            cancellation.obligation(),
            worth_store::physical_runtime::PhysicalEffectObligation::SettlementContinues
        );
        gate.release();
        execution.join().unwrap().unwrap()
    });

    assert_eq!(
        settled.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert_eq!(
        settled.signal(),
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::ReconciledFromPhysicalTruth,
        "a cancelled consumer cannot receive a committed Signal completion; \
         terminal physical settlement must reconcile from physical truth"
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before
                .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        1
    );
    let closed = serving.close();
    assert_eq!(
        closed
            .work()
            .drain()
            .continued_after_consumer_cancellation(),
        &[identity],
        "C5_PREDICATE:post-dispatch-cancellation: settlement must retain the losing consumer cancellation"
    );
}
