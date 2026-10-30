use worth_store::physical_runtime::{
    certification::CertificationPhysicalExecutionCheckpoint, PhysicalExecutorCommand,
    ServingPhysicalRuntime,
};

pub(super) fn assert_exact_write_settlement_order(
    serving: &ServingPhysicalRuntime,
    command: PhysicalExecutorCommand,
) {
    let checkpoint = serving.certification_pause_physical_execution_at(
        CertificationPhysicalExecutionCheckpoint::AfterExactWriteBeforeSchedulerSettlement,
    );
    let settled_before = serving.physical_work_observer().causal().records().len();
    let writes_before = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);

    std::thread::scope(|scope| {
        let execution = scope.spawn(|| serving.execute_physical_work(command));
        assert!(checkpoint.await_arrival());
        assert_eq!(
            serving
                .media_counters()
                .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
            writes_before + 1
        );
        assert_eq!(
            serving.physical_work_observer().causal().records().len(),
            settled_before
        );
        checkpoint.release();
        assert!(execution.join().unwrap().is_ok());
    });
    assert_eq!(
        serving.physical_work_observer().causal().records().len(),
        settled_before + 1
    );
}
