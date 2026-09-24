use worth_signal::facade::{ClockAdvanceRequest, ClockDomain, ClockTick, TemporalDuration};
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
use worth_store::physical_runtime::{
    PhysicalMutationCancellationOutcome, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationIndeterminateStage,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess,
    PhysicalMutationProvenNoEffectCause, PhysicalMutationRequest, RecordAppendBatch,
    RecordByteLimit, RecordReadLimits,
};

use super::super::super::{configuration, serving_from_initialization, serving_from_open};
use super::{completed, prepare};

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
        CertificationPhysicalMutationCheckpoint::BeforeEffectCutover,
    );
    let cancelled = prepare(&serving, placement, [0xA2; 32], b"together-cancel").start();
    assert!(pre_effect.await_arrival());
    assert!(matches!(
        cancelled.request_cancellation(),
        PhysicalMutationCancellationOutcome::AcceptedBeforeEffect { .. }
    ));
    pre_effect.release();
    match cancelled.wait() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal
            );
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("pre-effect cancellation must prove no effect")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("pre-effect cancellation became indeterminate")
        }
    }

    serving
        .certification_advance_physical_signal_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(7),
        ))
        .unwrap();
    match prepare_with_deadline(&serving, placement, [0xA3; 32], b"together-timeout", 7).execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal
        ),
        PhysicalMutationOutcome::Completed(_) | PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("timeout before seal must prove no effect")
        }
    }

    let post_effect = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterGroupSeal,
    );
    let effectful =
        prepare_with_deadline(&serving, placement, [0xA4; 32], b"together-effectful", 20).start();
    assert!(post_effect.await_arrival());
    serving
        .certification_advance_physical_signal_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(20),
        ))
        .unwrap();
    assert!(matches!(
        effectful.request_cancellation(),
        PhysicalMutationCancellationOutcome::SettlementAlreadyEffectful { .. }
    ));
    post_effect.release();
    let effectful = completed(effectful.wait());
    assert_eq!(effectful.persisted_records().len(), 1);

    let abandoned_gate = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let abandoned = prepare(&serving, placement, [0xA5; 32], b"together-abandoned").start();
    assert!(abandoned_gate.await_arrival());
    drop(abandoned);
    let closing = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::RuntimeClosingMarked,
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
        PhysicalMutationCancellationOutcome::StaleHandle { .. }
    ));
    match stale.wait() {
        PhysicalMutationOutcome::Indeterminate(fate) => {
            assert_eq!(
                fate.stage(),
                PhysicalMutationIndeterminateStage::RuntimeUnavailable
            );
        }
        PhysicalMutationOutcome::Completed(_) | PhysicalMutationOutcome::ProvenNoEffect(_) => {
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

fn prepare_with_deadline(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    record: &[u8],
    deadline_tick: u64,
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([record]).unwrap(),
            placement,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::at(
                    TemporalDuration::temporal_duration(deadline_tick).unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(
            prepared,
        )) => prepared,
        _ => panic!("deadline preparation must succeed"),
    }
}

fn execute_rewrite(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
) -> worth_store::physical_runtime::CompletedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .rewrite_selected_inline_segment(
            placement,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(
            prepared,
        )) => completed(prepared.execute()),
        worth_proof::TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(
            completed,
        )) => completed,
        _ => panic!("rewrite preparation must succeed or reconcile"),
    }
}

fn read_record(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    record: worth_store::physical_runtime::PhysicalRecordId,
    expected: &[u8],
) -> Vec<u8> {
    let reader = serving.records().unwrap();
    let mut session = reader
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(expected.len() as u32).unwrap()),
        )
        .unwrap();
    let mut bytes = vec![0_u8; expected.len()];
    let mut filled = 0;
    while filled < bytes.len() {
        let count = session.read_next(&mut bytes[filled..]).unwrap();
        assert!(count > 0);
        filled += count;
    }
    bytes
}
