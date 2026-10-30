#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::authority_mutants::*;

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write,
    worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_write_fault,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

#[test]
fn signal_evaluation_is_filesystem_effect_free() {
    let probe = std::env::temp_dir().join(format!(
        "worth-c5-signal-evaluation-effect-{}",
        std::process::id()
    ));
    if probe.exists() {
        std::fs::remove_file(&probe).unwrap();
    }
    let root = tempfile::tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let before = serving.media_counters();

    let admitted = admitted_write(&serving, request);

    assert!(
        !probe.exists() && serving.media_counters() == before,
        "C5_PREDICATE:signal-evaluation-effect: Signal evaluation produced a filesystem effect"
    );
    drop(admitted);
    serving.close();
}

#[test]
fn generic_signal_completion_cannot_upgrade_proven_no_effect() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_positioned_write_fault(
        root.path(),
        profile,
        worth_store_physical_backend::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::PermissionDenied,
            raw_os_error: None,
        },
    );
    let admitted = admitted_write(&serving, request);
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"noeffect".as_slice(),
    )
    .unwrap();
    let before = serving.media_counters();

    let outcome = serving.execute_physical_work(command).unwrap();

    assert_eq!(
        outcome.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::ProvenNoEffect
    );
    assert_eq!(
        outcome.signal(),
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::ReconciledFromPhysicalTruth,
        "C5_PREDICATE:generic-signal-settlement: generic completion upgraded physical no-effect truth"
    );
    let after = serving.media_counters();
    assert_eq!(
        after.completed_operations_for(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite
        ),
        before.completed_operations_for(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite
        )
    );
    assert_eq!(
        after
            .completed_bytes_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        before
            .completed_bytes_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
    );
    assert_eq!(
        after.denied_before_effect_for(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite
        ) - before.denied_before_effect_for(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite
        ),
        1
    );
    serving.close();
}

#[test]
fn physical_settlement_requires_backend_and_scheduler_evidence() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let admitted = admitted_write(&serving, request);
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"settled1".as_slice(),
    )
    .unwrap();

    let outcome = serving
        .execute_physical_work(command)
        .expect("the canonical write must reach physical settlement");

    assert_eq!(
        outcome.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted,
        "C5_PREDICATE:settlement: completed backend and scheduler evidence must produce exact physical settlement"
    );
    assert_eq!(
        outcome.signal(),
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::Committed
    );
    serving.close();
}

#[test]
fn derived_completion_must_join_the_real_signal_request() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let admitted = admitted_write(&serving, request);
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"derived1".as_slice(),
    )
    .unwrap();

    let outcome = serving
        .execute_physical_work(command)
        .expect("the canonical write must reach physical settlement");

    assert_eq!(
        outcome.signal(),
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::Committed
    );
    let signal = serving.physical_signal_observation().unwrap();
    assert_eq!(
        signal.active_locality_count(),
        0,
        "C5_PREDICATE:derived-completion: reported completion must retire Signal locality"
    );
    assert_eq!(
        signal.active_in_flight_count(),
        0,
        "C5_PREDICATE:derived-completion: reported completion must retire the Signal request"
    );
    serving.close();
}

#[test]
fn backend_receipts_cannot_settle_foreign_dispatched_work() {
    assert_cross_bound_receipts_are_rejected("backend-receipt");
}

#[test]
fn scheduler_counters_cannot_settle_cross_bound_backend_receipts() {
    assert_cross_bound_receipts_are_rejected("scheduler-counter-settlement");
}

#[test]
fn one_canonical_write_requires_one_backend_effect() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let admitted = admitted_write(&serving, request);
    let command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        admitted,
        b"onewrite".as_slice(),
    )
    .unwrap();
    let before = serving.media_counters();

    let outcome = serving.execute_physical_work(command);

    let outcome = outcome.expect("the admitted canonical write must reach settlement");
    assert_eq!(
        outcome.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted,
        "C5_PREDICATE:skipped-backend-write: canonical write skipped its backend effect"
    );
    let after = serving.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before
                .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        1,
        "C5_PREDICATE:raw-backend-dispatch: a write escaped the canonical executor path"
    );
    assert_eq!(
        after
            .completed_bytes_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            - before.completed_bytes_for(
                worth_store_physical_backend::MediaOperationRole::PositionedWrite
            ),
        8
    );
    serving.close();
}
