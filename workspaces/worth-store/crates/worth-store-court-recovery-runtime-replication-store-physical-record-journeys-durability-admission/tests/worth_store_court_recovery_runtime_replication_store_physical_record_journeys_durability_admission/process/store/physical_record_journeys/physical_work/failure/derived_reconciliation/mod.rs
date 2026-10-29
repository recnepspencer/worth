use {tempfile::tempdir};
use {worth_store::physical_runtime::PhysicalExecutorCommand};
use {worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write, worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_write_fault_at, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_mutation_fixture, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile};

#[test]
fn later_batch_panic_retains_earlier_settlement_without_repeating_media() {
    let retry_probe = std::env::temp_dir().join(format!(
        "worth-c5-physical-effect-retry-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&retry_probe);
    let root = tempdir().unwrap();
    let (profile, first_request, second_request) = disjoint_mutation_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_positioned_write_fault_at(
        root.path(),
        profile,
        2,
        worth_store_physical_backend::MediaFaultDirective::PanicAfter,
    );
    let first = admitted_write(&serving, first_request);
    let first_identity = first.intent().identity();
    let second = admitted_write(&serving, second_request);
    let commands = vec![
        worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(first, b"first001".as_slice()).unwrap(),
        worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(second, b"second02".as_slice()).unwrap(),
    ]
    .into_boxed_slice();
    let before = serving.media_counters();
    let media_observer = serving.observer();
    let observer = serving.physical_work_observer();

    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = serving.execute_physical_work_batch(commands);
    }));

    assert!(panic.is_err());
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        2
    );
    assert_eq!(
        &std::fs::read(root.path().join("families/records/bootstrap.catalog")).unwrap()[8..16],
        b"first001",
        "the earlier command must have produced its real media effect"
    );
    let causal = observer.causal().records();
    let first_record = causal
        .iter()
        .find(|record| record.identity() == first_identity)
        .expect("the earlier physical settlement must remain causally recorded");
    assert_eq!(first_record.derived_completion(), None);

    let closed = serving.close_plan().execute();

    assert!(closed.requires_inspection());
    assert!(
        closed
            .shutdown()
            .work()
            .drain()
            .derived_reconciliation_deferred()
            .is_empty(),
        "close must reconcile the retained completion instead of merely reporting it"
    );
    let reconciled_records = observer.causal().records();
    let reconciled = reconciled_records
        .iter()
        .find(|record| record.identity() == first_identity)
        .expect("reconciled work must remain causally observable");
    assert_eq!(
        reconciled.derived_completion(),
        Some(
            worth_store::physical_runtime::PhysicalSignalSettlementOutcome::
                ReconciledFromPhysicalTruth
        )
    );
    assert_eq!(
        media_observer
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        2,
        "C5_PREDICATE:physical-effect-no-retry: derived reconciliation must never repeat either filesystem effect"
    );
    assert!(
        !retry_probe.exists(),
        "C5_PREDICATE:physical-effect-no-retry: derived reconciliation reacquired filesystem effect authority"
    );
    assert_eq!(
        closed.shutdown().signal(),
        worth_store::physical_runtime::PhysicalSignalShutdownOutcome::Disposed
    );
}
