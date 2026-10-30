#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::execution_capability::*;
use tempfile::tempdir;
use worth_store_physical_backend::MediaOperationRole;
use {
    worth_store::physical_runtime::PhysicalExecutorCommand,
    worth_store::physical_runtime::PhysicalStoreCloseOutcome,
    worth_store::physical_runtime::PhysicalStoreClosePhase,
    worth_store::physical_runtime::PhysicalWorkPreEffectDenial,
};

use worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write;
use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_mutation_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
};

#[test]
fn stale_execution_capability_denies_every_batch_command_before_effects() {
    let root = tempdir().unwrap();
    let (profile, first_request, second_request) = disjoint_mutation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let execution = serving.certification_stale_physical_work_execution();
    let first = admitted_write(&serving, first_request);
    let second = admitted_write(&serving, second_request);
    let identities = [first.intent().identity(), second.intent().identity()];
    let commands = write_commands(first, second);
    let before = serving.media_counters();
    let signal_gate = serving.certification_pause_physical_signal_after_dequeue();

    let outcome = execution.execute_physical_work_batch(commands);

    assert!(
        outcome.executions().is_empty(),
        "C5_PREDICATE:stale-generation: a stale execution capability must produce no effects"
    );
    assert_eq!(
        outcome
            .denied_before_effect()
            .iter()
            .map(|denial| (denial.identity(), denial.denial()))
            .collect::<Vec<_>>(),
        identities
            .into_iter()
            .map(|identity| (
                identity,
                worth_store::physical_runtime::PhysicalWorkPreEffectDenial::StaleGeneration
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(serving.media_counters(), before);
    assert!(
        signal_gate.await_arrivals(1),
        "Signal did not dequeue the stale command's abandonment"
    );
    let close = serving.close_plan();
    let progress = close.observation();
    let closed = std::thread::scope(|scope| {
        let closing = scope.spawn(move || close.execute());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !progress
            .reached(worth_store::physical_runtime::PhysicalStoreClosePhase::AdmissionStopped)
            && std::time::Instant::now() < deadline
        {
            std::thread::yield_now();
        }
        assert!(progress
            .reached(worth_store::physical_runtime::PhysicalStoreClosePhase::AdmissionStopped));
        assert!(!progress.reached(
            worth_store::physical_runtime::PhysicalStoreClosePhase::DispatchSettlementComplete
        ));
        signal_gate.release();
        closing.join().unwrap()
    });
    assert!(matches!(
        closed,
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
    assert_eq!(
        closed.shutdown().work().drain().released_before_dispatch(),
        identities
    );
    assert_eq!(
        closed
            .shutdown()
            .signal_summary()
            .unwrap()
            .active_in_flight_node_count(),
        0
    );
}

#[test]
fn stale_execution_capability_cannot_cross_the_real_effect_boundary() {
    let root = tempdir().unwrap();
    let (profile, request, _) = disjoint_mutation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let execution = serving.certification_stale_physical_work_execution();
    let admitted = admitted_write(&serving, request);
    let identity = admitted.intent().identity();
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"stale001".as_slice(),
    )
    .unwrap();
    let before = serving.media_counters();

    let outcome = execution.execute_physical_work_batch(vec![command].into_boxed_slice());

    assert!(
        outcome.executions().is_empty(),
        "C5_PREDICATE:stale-generation: stale execution authority crossed the effect boundary"
    );
    assert_eq!(
        outcome
            .denied_before_effect()
            .iter()
            .map(|denial| (denial.identity(), denial.denial()))
            .collect::<Vec<_>>(),
        vec![(
            identity,
            worth_store::physical_runtime::PhysicalWorkPreEffectDenial::StaleGeneration
        )]
    );
    assert_eq!(serving.media_counters(), before);
    serving.close();
}

#[test]
fn released_runtime_reports_every_batch_command_as_admission_stopped() {
    let root = tempdir().unwrap();
    let (profile, first_request, second_request) = disjoint_mutation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let execution = serving.physical_work_execution();
    let first = admitted_write(&serving, first_request);
    let second = admitted_write(&serving, second_request);
    let identities = [first.intent().identity(), second.intent().identity()];
    let commands = write_commands(first, second);
    let media_observer = serving.observer();
    let work_observer = serving.physical_work_observer();

    drop(serving);
    let before = media_observer.media_counters();
    let outcome = execution.execute_physical_work_batch(commands);

    assert!(outcome.executions().is_empty());
    assert_eq!(
        outcome
            .denied_before_effect()
            .iter()
            .map(|denial| (denial.identity(), denial.denial()))
            .collect::<Vec<_>>(),
        identities
            .into_iter()
            .map(|identity| (
                identity,
                worth_store::physical_runtime::PhysicalWorkPreEffectDenial::AdmissionStopped
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(media_observer.media_counters(), before);
    let terminal = work_observer
        .terminal()
        .expect("runtime release must publish terminal physical-work truth");
    assert_eq!(terminal.declared(), identities.len() as u64);
    assert_eq!(
        terminal
            .terminal()
            .iter()
            .map(|work| work.identity())
            .collect::<Vec<_>>(),
        identities
    );
    assert_eq!(terminal.residual(), 0);
    assert_eq!(terminal.unaccounted_terminal(), 0);
}

#[test]
fn abandonment_worker_failure_closes_with_exact_inspection_evidence() {
    let root = tempdir().unwrap();
    let (profile, first_request, second_request) = disjoint_mutation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let first = admitted_write(&serving, first_request);
    let second = admitted_write(&serving, second_request);
    let identities = [first.intent().identity(), second.intent().identity()];
    let media_observer = serving.observer();
    let before = media_observer.media_counters();
    serving.certification_fail_next_physical_signal_abandonment();

    drop(first);
    drop(second);

    let close = serving.close_plan();
    let (finished, outcome) = std::sync::mpsc::sync_channel(1);
    let closing = std::thread::spawn(move || finished.send(close.execute()).unwrap());
    let closed = outcome
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("worker failure must not strand the bounded abandonment join");
    closing.join().unwrap();
    assert!(closed.requires_inspection());
    let after = media_observer.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
    );
    assert_eq!(after.requested_bytes(), before.requested_bytes());
    assert_eq!(after.completed_bytes(), before.completed_bytes());
    let drain = closed.shutdown().work().drain();
    let mut terminal = drain.released_before_dispatch().to_vec();
    terminal.extend_from_slice(drain.cancelled_before_dispatch());
    terminal.sort_by_key(|identity| identity.operation().get());
    assert_eq!(terminal, identities);
    assert_eq!(drain.exact_identity_count(), identities.len());
    assert_eq!(closed.shutdown().work().residual(), 0);
}
