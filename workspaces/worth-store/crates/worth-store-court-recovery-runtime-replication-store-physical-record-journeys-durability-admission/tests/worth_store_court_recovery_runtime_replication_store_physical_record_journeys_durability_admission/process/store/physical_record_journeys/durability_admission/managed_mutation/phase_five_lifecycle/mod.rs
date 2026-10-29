#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::phase_five_lifecycle::*;
use {worth_signal::facade::ClockAdvanceRequest, worth_signal::facade::ClockDomain, worth_signal::facade::ClockTick, worth_signal::facade::TemporalDuration};
use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint};
use {worth_store::physical_runtime::PhysicalMutationCancellationOutcome, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationIndeterminateStage, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization, worth_store_testkit_store::physical_record_journeys::serving_from_open};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare};

#[test]
fn cancellation_timeout_abandonment_close_and_reopen_settle_once() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let durable =
        completed(prepare(&serving, placement, [0xA1; 32], b"together-durable").execute());
    let identity = durable.persisted_records()[0];
    let record = durable.into_acknowledgment().record_ids().next().unwrap();

    let pre_effect = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeEffectCutover,
    );
    let cancelled = prepare(&serving, placement, [0xA2; 32], b"together-cancel").start();
    assert!(pre_effect.await_arrival());
    assert!(matches!(
        cancelled.request_cancellation(),
        worth_store::physical_runtime::PhysicalMutationCancellationOutcome::AcceptedBeforeEffect { .. }
    ));
    pre_effect.release();
    match cancelled.wait() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal
            );
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {
            panic!("pre-effect cancellation must prove no effect")
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("pre-effect cancellation became indeterminate")
        }
    }

    serving
        .certification_advance_physical_signal_clock(worth_signal::facade::ClockAdvanceRequest::new(
            worth_signal::facade::ClockDomain::MonotonicExecution,
            worth_signal::facade::ClockTick::new(7),
        ))
        .unwrap();
    match prepare_with_deadline(&serving, placement, [0xA3; 32], b"together-timeout", 7).execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal
        ),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) | worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("timeout before seal must prove no effect")
        }
    }

    let post_effect = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterGroupSeal,
    );
    let effectful =
        prepare_with_deadline(&serving, placement, [0xA4; 32], b"together-effectful", 20).start();
    assert!(post_effect.await_arrival());
    serving
        .certification_advance_physical_signal_clock(worth_signal::facade::ClockAdvanceRequest::new(
            worth_signal::facade::ClockDomain::MonotonicExecution,
            worth_signal::facade::ClockTick::new(20),
        ))
        .unwrap();
    assert!(matches!(
        effectful.request_cancellation(),
        worth_store::physical_runtime::PhysicalMutationCancellationOutcome::SettlementAlreadyEffectful { .. }
    ));
    post_effect.release();
    let effectful = completed(effectful.wait());
    assert_eq!(effectful.persisted_records().len(), 1);

    let abandoned_gate = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let abandoned = prepare(&serving, placement, [0xA5; 32], b"together-abandoned").start();
    assert!(abandoned_gate.await_arrival());
    drop(abandoned);
    let closing = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::RuntimeClosingMarked,
    );
    let stale = prepare(&serving, placement, [0xA6; 32], b"together-stale");
    let observer = serving.observer();
    let close = std::thread::spawn(move || serving.close());
    assert!(
        closing.await_arrival(),
        "close must reach the drain boundary while the dropped mutation is still held"
    );
    assert!(
        !close.is_finished(),
        "close must stay pending at the drain boundary until settlement is released"
    );
    abandoned_gate.release();
    assert!(
        !close.is_finished(),
        "releasing settlement must not finish close while the drain boundary is held"
    );
    closing.release();
    let shutdown = close.join().expect("close must finish after settlement");
    assert!(shutdown.mutations().cancellation_effectful() >= 1);
    assert!(shutdown.mutations().proven_no_effect() >= 2);
    assert_eq!(shutdown.mutations().completed_unobserved(), 1);
    assert_eq!(shutdown.mutations().indeterminate(), 0);

    let stale = stale.start();
    assert!(matches!(
        stale.request_cancellation(),
        worth_store::physical_runtime::PhysicalMutationCancellationOutcome::StaleHandle { .. }
    ));
    match stale.wait() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            assert_eq!(
                fate.stage(),
                worth_store::physical_runtime::PhysicalMutationIndeterminateStage::RuntimeUnavailable
            );
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) | worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(_) => {
            panic!("a completion after close must stay stale")
        }
    }

    let serving = serving_from_open(&root);
    assert!(
        observer.acquisition_snapshot().is_err(),
        "the abandoned observer must not observe the fresh incarnation"
    );
    assert_eq!(
        read_record(&serving, record, b"together-durable"),
        b"together-durable"
    );
    let rewritten = execute_rewrite(&serving, placement, [0xA7; 32]);
    assert!(rewritten.persisted_records().contains(&identity));
    let generation = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    let retried = execute_rewrite(&serving, placement, [0xA7; 32]);
    assert_eq!(retried.mutation_identity(), rewritten.mutation_identity());
    assert!(retried.persisted_records().contains(&identity));
    assert_eq!(
        serving
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .root_generation(),
        generation,
        "retrying the published rewrite must not publish another root"
    );
    assert_eq!(
        read_record(&serving, record, b"together-durable"),
        b"together-durable"
    );
    serving.close();
}
