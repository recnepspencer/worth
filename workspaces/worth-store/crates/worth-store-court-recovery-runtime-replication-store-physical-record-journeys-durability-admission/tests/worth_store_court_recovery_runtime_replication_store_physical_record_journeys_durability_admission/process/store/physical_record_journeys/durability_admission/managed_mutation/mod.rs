#[allow(unused_imports)] use worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::*;
use {worth_proof::TransitionOutcome};
use {worth_signal::facade::TemporalDuration};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint};
use {worth_store::physical_runtime::PhysicalMutationCancellationOutcome, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalMutationTerminalObservation, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch};


















#[test]
fn managed_mutation_completion_is_the_only_acknowledgment_source() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let prepared = prepare(&serving, placement, [181; 32], b"managed-completion");

    let identity = prepared.mutation_identity();
    let completed = match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(completed) => completed,
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!(
                "managed mutation unexpectedly proved no effect: {:?}",
                fate.cause()
            )
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!(
                "managed mutation became indeterminate at {:?}",
                fate.stage()
            )
        }
    };
    assert_eq!(completed.mutation_identity(), identity);
    assert_eq!(completed.completed_breadth().record_count(), 1);
    assert_eq!(completed.persisted_records().len(), 1);
    let acknowledgment = completed.into_acknowledgment();
    assert_eq!(acknowledgment.mutation_identity(), identity);
    assert_eq!(acknowledgment.completed_breadth().record_count(), 1);
    assert_eq!(acknowledgment.persisted_records().len(), 1);
    let executed = acknowledgment.executed_boundary_evidence();
    assert_eq!(executed.mutation_identity(), identity);
    assert_eq!(executed.completed_breadth().record_count(), 1);
    let performance = acknowledgment.performance_evidence();
    assert_eq!(performance.mutation_identity(), identity);
    assert_eq!(performance.bytes_completed(), performance.bytes_requested());

    let shutdown = serving.close();
    assert_eq!(shutdown.mutations().started(), 1);
    assert_eq!(shutdown.mutations().completed(), 1);
    assert_eq!(shutdown.mutations().proven_no_effect(), 0);
    assert_eq!(shutdown.mutations().indeterminate(), 0);
}

#[test]
fn duplicate_preparations_join_one_store_owned_attempt() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let key_material = [182; 32];
    let first = prepare(&serving, placement, key_material, b"managed-duplicate");
    let duplicate = prepare(&serving, placement, key_material, b"managed-duplicate");
    assert_eq!(first.mutation_identity(), duplicate.mutation_identity());

    let first = first.start();
    let duplicate = duplicate.start();
    let first = completed(first.wait());
    let duplicate = completed(duplicate.wait());
    assert_eq!(first.mutation_identity(), duplicate.mutation_identity());
    assert_eq!(first.completed_breadth(), duplicate.completed_breadth());

    let shutdown = serving.close();
    assert_eq!(shutdown.mutations().started(), 1);
    assert_eq!(shutdown.mutations().completed(), 1);
}

#[test]
fn dropped_handle_abandons_observation_while_close_drains_settlement() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let prepared = prepare(&serving, placement, [183; 32], b"managed-fire-and-forget");

    let handle = prepared.start();
    let progress_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < progress_deadline {
        if !matches!(
            handle.progress().phase(),
            worth_store::physical_runtime::PhysicalMutationProgressPhase::Admitted
                | worth_store::physical_runtime::PhysicalMutationProgressPhase::WalAppend
        ) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(!matches!(
        handle.progress().phase(),
        worth_store::physical_runtime::PhysicalMutationProgressPhase::Admitted
            | worth_store::physical_runtime::PhysicalMutationProgressPhase::WalAppend
    ));
    drop(handle);
    let shutdown = serving.close();
    assert_eq!(shutdown.mutations().started(), 1);
    assert_eq!(shutdown.mutations().completed(), 1);
    assert_eq!(shutdown.mutations().completed_unobserved(), 1);
    assert_eq!(shutdown.mutations().indeterminate(), 0);
}

#[test]
fn cancellation_before_group_seal_is_accepted_and_proves_no_effect() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let gate = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeEffectCutover,
    );
    let handle = prepare(&serving, placement, [184; 32], b"cancel-before-seal").start();
    assert!(gate.await_arrival());
    assert!(matches!(
        handle.request_cancellation(),
        worth_store::physical_runtime::PhysicalMutationCancellationOutcome::AcceptedBeforeEffect { .. }
    ));
    gate.release();
    match handle.wait() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal
            );
            assert_eq!(
                fate.diagnostic_evidence().mutation_identity(),
                fate.mutation_identity()
            );
        }
        _ => panic!("accepted pre-effect cancellation must prove no effect"),
    }

    assert_eq!(
        serving
            .physical_mutation_observation()
            .cancellation_accepted(),
        1
    );

    let shutdown = serving.close();
    assert_eq!(shutdown.mutations().proven_no_effect(), 1);
}

#[test]
fn deadline_uses_the_authoritative_signal_clock() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let prepared = prepare_with_deadline(&serving, placement, [185; 32], b"signal-deadline", 7);
    serving
        .certification_advance_physical_signal_clock(
            worth_signal::facade::ClockAdvanceRequest::new(
                worth_signal::facade::ClockDomain::MonotonicExecution,
                worth_signal::facade::ClockTick::new(7),
            ),
        )
        .unwrap();

    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal
        ),
        _ => panic!("elapsed Signal deadline must prove no effect before group seal"),
    }
    assert_eq!(serving.close().mutations().proven_no_effect(), 1);
}

#[test]
fn cancellation_after_group_seal_is_effectful_and_terminal_is_stable() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let gate = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterGroupSeal,
    );
    let first = prepare(&serving, placement, [186; 32], b"cancel-after-seal");
    let duplicate = prepare(&serving, placement, [186; 32], b"cancel-after-seal");
    let first = first.start();
    let duplicate = duplicate.start();
    assert!(gate.await_arrival());
    assert!(matches!(
        duplicate.request_cancellation(),
        worth_store::physical_runtime::PhysicalMutationCancellationOutcome::SettlementAlreadyEffectful { .. }
    ));
    gate.release();
    let identity = completed(first.wait()).mutation_identity();
    assert!(matches!(
        duplicate.request_cancellation(),
        worth_store::physical_runtime::PhysicalMutationCancellationOutcome::AlreadyTerminal(
            worth_store::physical_runtime::PhysicalMutationTerminalObservation::Completed(observed)
        ) if observed == identity
    ));
    let observation = serving.physical_mutation_observation();
    assert_eq!(observation.cancellation_effectful(), 1);
    assert_eq!(observation.cancellation_terminal(), 1);
    assert_eq!(completed(duplicate.wait()).mutation_identity(), identity);
    assert_eq!(serving.close().mutations().completed(), 1);
}

#[test]
fn prepared_mutation_started_after_owner_release_is_explicitly_stale() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let prepared = prepare(&serving, placement, [187; 32], b"stale-start-port");
    assert_eq!(serving.close().mutations().started(), 0);

    let handle = prepared.start();
    assert!(matches!(
        handle.request_cancellation(),
        worth_store::physical_runtime::PhysicalMutationCancellationOutcome::StaleHandle { .. }
    ));
    match handle.wait() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            assert_eq!(
                fate.stage(),
                worth_store::physical_runtime::PhysicalMutationIndeterminateStage::RuntimeUnavailable
            );
        }
        _ => panic!("released mutation owner must produce stale indeterminate observation"),
    }
}

#[test]
fn cancellation_during_runtime_close_is_explicit_and_close_drains_the_attempt() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let gate = serving.certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeEffectCutover,
    );
    let handle = prepare(&serving, placement, [188; 32], b"cancel-during-close").start();
    assert!(gate.await_arrival());

    let close = std::thread::spawn(move || serving.close());
    let closing_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !handle.progress().runtime_closing() && std::time::Instant::now() < closing_deadline {
        std::thread::yield_now();
    }
    assert!(handle.progress().runtime_closing());
    assert!(matches!(
        handle.request_cancellation(),
        worth_store::physical_runtime::PhysicalMutationCancellationOutcome::RuntimeClosing { .. }
    ));

    gate.release();
    match handle.wait() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal
        ),
        _ => panic!("close before effect cutover must prove no effect"),
    }
    let shutdown = close.join().expect("close thread must not panic");
    assert_eq!(shutdown.mutations().started(), 1);
    assert_eq!(shutdown.mutations().proven_no_effect(), 1);
    assert_eq!(shutdown.mutations().cancellation_runtime_closing(), 1);
}
#[path = "candidate_verification/mod.rs"]
pub(crate) mod candidate_verification;
#[path = "extent_record_rewrite/mod.rs"]
pub(crate) mod extent_record_rewrite;
#[path = "phase_five_lifecycle/mod.rs"]
pub(crate) mod phase_five_lifecycle;
#[path = "selected_segment_rewrite/mod.rs"]
pub(crate) mod selected_segment_rewrite;
