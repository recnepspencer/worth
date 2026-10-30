mod settlement_order;

use tempfile::tempdir;
#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::physical_work::executor::*;

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_mutation_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::family_locality_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};
use {
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_read_work,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_work,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::write_demand,
    worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt,
    worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt_for,
};

#[test]
fn write_settles_physical_truth_before_signal_completion() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let demand = write_demand(&serving, ready_work(&serving, mutation_request));
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let demand = secure_demand(demand, &backend);
    let admitted = serving
        .admit_physical_scheduler_demand(demand, &backend, policy_receipt(requested_budget))
        .unwrap();
    let consumer = admitted.consumer_handle();
    let before = serving.media_counters();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"executed".as_slice(),
    )
    .unwrap();

    let outcome = serving.execute_physical_work(command).unwrap();

    assert_eq!(
        outcome.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert_eq!(
        outcome.signal(),
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::Committed
    );
    assert_eq!(outcome.settled().intent().identity(), consumer.identity());
    let after = serving.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before
                .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        1
    );
    serving.close();
}

#[test]
fn exact_write_checkpoint_precedes_scheduler_and_signal_settlement() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let admitted = admitted_write(&serving, mutation_request);
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"executed".as_slice(),
    )
    .unwrap();
    settlement_order::assert_exact_write_settlement_order(&serving, command);
    serving.close();
}

#[test]
fn read_returns_the_exact_bounded_destination() {
    let root = tempdir().unwrap();
    let (profile, read_request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let ready = ready_read_work(&serving, read_request);
    let demand = worth_store::physical_runtime::PhysicalSchedulerDemand::foreground(
        ready,
        worth_store_testkit_store::physical_work::scheduler_reservation::reserved_buffered_file_read(&serving),
        None,
    )
    .unwrap();
    let work = demand.queue_work();
    let backend_requirement = work.backend_requirement();
    let requested_budget = work.requested_budget();
    let backend = serving
        .admit_physical_scheduler_capability(backend_requirement)
        .unwrap();
    let admitted = serving
        .admit_physical_scheduler_demand(
            demand,
            &backend,
            policy_receipt_for(
                requested_budget,
                0,
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeRead,
            ),
        )
        .unwrap();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::read(admitted).unwrap();

    let outcome = serving.execute_physical_work(command).unwrap();

    assert_eq!(
        outcome.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::ReadCompleted
    );
    assert_eq!(outcome.settled().evidence().completed_payload_bytes(), 8);
    assert_eq!(
        outcome.signal(),
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::Committed
    );
    serving.close();
}

#[test]
fn cancellation_before_dispatch_revokes_the_command_without_an_effect() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let admitted = admitted_write(&serving, mutation_request);
    let consumer = admitted.consumer_handle();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"cancelld".as_slice(),
    )
    .unwrap();
    let before = serving.media_counters();

    let cancellation = serving.cancel_physical_work(consumer).unwrap();

    assert!(cancellation.signal().cancelled_request().is_some());
    assert_eq!(
        cancellation.obligation(),
        worth_store::physical_runtime::PhysicalEffectObligation::NotDispatched
    );
    assert!(matches!(
        serving.execute_physical_work(command),
        Err(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::ConsumerCancelled)
    ));
    assert_eq!(serving.media_counters(), before);
    let closed = serving.close();
    assert_eq!(
        closed.work().drain().cancelled_before_dispatch(),
        &[consumer.identity()]
    );
}

#[test]
fn signal_timeout_uses_deterministic_clock_and_proves_no_dispatch() {
    let root = tempdir().unwrap();
    let (profile, _, mutation_request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let admitted = admitted_write(&serving, mutation_request);
    let consumer = admitted.consumer_handle();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"timedout".as_slice(),
    )
    .unwrap();
    let before = serving.media_counters();
    serving
        .advance_physical_signal_clock(
            consumer,
            worth_signal::facade::ClockAdvanceRequest::new(
                worth_signal::facade::ClockDomain::MonotonicExecution,
                worth_signal::facade::ClockTick::new(1_000),
            ),
        )
        .unwrap();

    let timeout = serving.timeout_physical_work(consumer).unwrap();

    assert!(timeout.signal().timed_out_request().is_some());
    assert_eq!(
        timeout.obligation(),
        worth_store::physical_runtime::PhysicalEffectObligation::NotDispatched,
        "C5_PREDICATE:store-local-async-registry: local timeout state overrode Signal's effect obligation"
    );
    assert!(matches!(
        serving.execute_physical_work(command),
        Err(worth_store::physical_runtime::PhysicalWorkPreEffectDenial::ConsumerCancelled)
    ));
    assert_eq!(serving.media_counters(), before);
    let closed = serving.close();
    assert_eq!(
        closed.work().drain().cancelled_before_dispatch(),
        &[consumer.identity()]
    );
}

#[test]
fn bounded_completion_batch_preserves_each_physical_identity() {
    let root = tempdir().unwrap();
    let (profile, first_request, second_request) = disjoint_mutation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let first = admitted_write(&serving, first_request);
    let second = admitted_write(&serving, second_request);
    let identities = [first.intent().identity(), second.intent().identity()];
    let before = serving.media_counters();
    let commands = vec![
        worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
            first,
            b"first001".as_slice(),
        )
        .unwrap(),
        worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
            second,
            b"second02".as_slice(),
        )
        .unwrap(),
    ]
    .into_boxed_slice();

    let batch = serving.execute_physical_work_batch(commands);

    assert!(batch.denied_before_effect().is_empty());
    assert_eq!(batch.executions().len(), 2);
    let signal_outcomes = batch
        .executions()
        .iter()
        .map(|execution| execution.signal())
        .collect::<Vec<_>>();
    assert!(
        signal_outcomes
            .iter()
            .all(|outcome| *outcome != worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable),
        "batch Signal outcomes: {signal_outcomes:?}"
    );
    assert_eq!(
        signal_outcomes
            .iter()
            .filter(|outcome| **outcome
                == worth_store::physical_runtime::PhysicalSignalSettlementOutcome::Committed)
            .count(),
        1
    );
    assert_eq!(
        signal_outcomes
            .iter()
            .filter(|outcome| {
                **outcome == worth_store::physical_runtime::PhysicalSignalSettlementOutcome::ReconciledFromPhysicalTruth
            })
            .count(),
        1
    );
    assert_eq!(
        batch
            .executions()
            .iter()
            .map(|execution| execution.settled().intent().identity())
            .collect::<Vec<_>>(),
        identities
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before
                .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        2
    );
    assert_eq!(
        serving.physical_work_counters().count(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        2
    );
    assert_eq!(
        serving.physical_work_counters().count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite,
            worth_store::physical_runtime::PhysicalWorkPressureClass::ForegroundMutation,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        2
    );

    let closed = serving.close_plan().execute();
    assert!(
        matches!(closed, worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }),
        "close inspection: records={:?}, residency={}, drain={:?}, signal={:?}, cancellations={}, summary={:?}, media={:?}",
        closed.shutdown().records().posture(),
        closed.shutdown().residency().requires_inspection(),
        closed.shutdown().work().drain(),
        closed.shutdown().signal(),
        closed.shutdown().signal_cancellation_failures(),
        closed.shutdown().signal_summary(),
        closed.shutdown().media().release(),
    );
    assert_eq!(closed.shutdown().work().drain().settled(), identities);
    assert_eq!(
        closed
            .shutdown()
            .signal_summary()
            .unwrap()
            .active_in_flight_node_count(),
        0
    );
}

#[path = "mixed_route_batch.rs"]
mod mixed_route_batch;
