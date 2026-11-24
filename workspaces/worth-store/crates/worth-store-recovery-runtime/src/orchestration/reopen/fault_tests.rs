//! Regression tests for Issue #355: media faults during `execute_fresh_reopen`
//! must settle cleanly and return typed `PublicationIndeterminate` without
//! panicking on `shutdown_is_quiescent()`, leaving the store cleanly recoverable.
use std::num::NonZeroU64;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::certification::{
    CertificationMediaFaultActivation, MediaFaultDirective, MediaOperationRole,
};
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalRecoveryFreshReopenDenialKind, PhysicalRecoveryFreshReopenStage,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    PhysicalResidencyStoreWorld,
};

use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryOutcome, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryStaticConfiguration,
};

const RECOVERY_MEMORY_BYTES: u64 = 16 << 20;

#[test]
fn fresh_reopen_media_faults_return_typed_publication_indeterminate_and_recover_cleanly() {
    std::thread::Builder::new()
        .name("fresh-reopen-media-fault-regression".to_owned())
        .stack_size(16 << 20)
        .spawn(run_fresh_reopen_media_fault_cases)
        .unwrap()
        .join()
        .unwrap();
}

fn run_fresh_reopen_media_fault_cases() {
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("fresh-reopen-media-fault").unwrap();
    let _ack =
        canonical_physical_mutation_acknowledgment(&world, [0x41; 32], b"reopen-fault-covered");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x42; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("checkpoint must admit");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_durable_wal_attempt_without_execution(&world, [0x43; 32], b"reopen-fault-tail");
    let retained = world.retained_root();
    let root = retained.path();
    drop(world);

    // 1. PositionedRead (identified operation) faults armed immediately before `durable.reopen()`
    //    hit both CurrentSelector (#1) and RootManifest (#2), and recovery deterministically waits
    //    when abandonment is held pending at the Signal worker dequeue gate instead of panicking
    //    on `shutdown_is_quiescent()`.
    for (nth, expected_stage) in [
        (1, PhysicalRecoveryFreshReopenStage::CurrentSelector),
        (2, PhysicalRecoveryFreshReopenStage::RootManifest),
    ] {
        assert_fresh_reopen_waits_for_held_abandonment(root, nth, expected_stage);
    }

    // 2. OpenExisting (raw role ordinal) faults across recovery and `durable.reopen()` hit both
    //    CurrentSelector and RootManifest without panicking on `shutdown_is_quiescent()`.
    let mut observed_selector = false;
    let mut observed_root = false;
    for ordinal in 1..=256u64 {
        let Ok(request) =
            try_faulted_raw_ordinal_request(root, MediaOperationRole::OpenExisting, ordinal)
        else {
            continue;
        };
        let Ok(durable) = try_advance_to_durable(request) else {
            continue;
        };
        match durable.reopen() {
            Ok(reopened) => {
                drop(reopened);
                break;
            }
            Err(PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate)) => {
                let failure = indeterminate
                    .reopen_failure()
                    .expect("reopen failure evidence must be attached");
                assert!(matches!(
                    failure.denial().kind(),
                    PhysicalRecoveryFreshReopenDenialKind::Media(_)
                ));
                match failure.denial().stage() {
                    PhysicalRecoveryFreshReopenStage::CurrentSelector => observed_selector = true,
                    PhysicalRecoveryFreshReopenStage::RootManifest => observed_root = true,
                    stage => panic!("OpenExisting#{ordinal}: unexpected stage {stage:?}"),
                }
                drop(indeterminate);
                assert_clean_recovery(root, "OpenExisting", ordinal);
            }
            Err(other) => {
                panic!("OpenExisting#{ordinal}: unexpected reopen outcome {other:?}");
            }
        }
    }
    assert!(
        observed_selector && observed_root,
        "OpenExisting faults must cover both CurrentSelector and RootManifest"
    );
}

fn assert_fresh_reopen_waits_for_held_abandonment(
    root: &Path,
    nth: u64,
    expected_stage: PhysicalRecoveryFreshReopenStage,
) {
    let (request, activation) =
        faulted_identified_request(root, MediaOperationRole::PositionedRead, nth);
    let durable = advance_to_durable(request);
    let abandonment_gate = durable
        .payload()
        .state
        .coordination
        .owner()
        .certification_pause_abandonment_after_dequeue();
    let wait_observer = durable
        .payload()
        .state
        .coordination
        .owner()
        .certification_abandonment_wait_observer();
    activation.arm().unwrap();

    let (tx, rx) = mpsc::channel();
    let worker = std::thread::Builder::new()
        .name(format!("fresh-reopen-held-abandonment-{nth}"))
        .stack_size(16 << 20)
        .spawn(move || {
            let _ = tx.send(durable.reopen());
        })
        .unwrap();

    assert!(
        abandonment_gate.await_arrivals(1),
        "PositionedRead#{nth}: Signal worker must dequeue and hold the retry-safe abandonment"
    );
    assert!(
        wait_observer.await_observed_outstanding(1),
        "PositionedRead#{nth}: recovery must enter await_abandonment_idle() while abandonment is outstanding"
    );
    assert!(
        matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)),
        "PositionedRead#{nth}: recovery must remain blocked until abandonment completes"
    );
    abandonment_gate.release();

    let outcome = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("PositionedRead: reopen must complete after abandonment gate release");
    worker.join().unwrap();
    let Err(PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate)) = outcome else {
        panic!("PositionedRead#{nth}: fresh reopen fault must return PublicationIndeterminate");
    };
    assert!(
        activation.is_consumed(),
        "PositionedRead#{nth}: fault activation must fire during fresh reopen"
    );
    let failure = indeterminate
        .reopen_failure()
        .expect("reopen failure evidence must be attached");
    assert_eq!(failure.denial().stage(), expected_stage);
    assert!(matches!(
        failure.denial().kind(),
        PhysicalRecoveryFreshReopenDenialKind::Media(_)
    ));
    drop(indeterminate);
    assert_clean_recovery(root, "PositionedRead", nth);
}

fn advance_to_durable(
    request: PhysicalRecoveryOpenRequest,
) -> crate::progression::NamespaceDurablePhysicalRecovery {
    try_advance_to_durable(request).expect("unfaulted advance to durable succeeds")
}

fn try_advance_to_durable(
    request: PhysicalRecoveryOpenRequest,
) -> Result<crate::progression::NamespaceDurablePhysicalRecovery, ()> {
    let admitted = request.admit().map_err(|_| ())?;
    let discovered = admitted.discover().map_err(|_| ())?;
    let selected = discovered.select().map_err(|_| ())?;
    let planned = selected.plan().map_err(|_| ())?;
    let staged = planned.stage().map_err(|_| ())?;
    staged.publish().map_err(|_| ())
}

fn assert_clean_recovery(root: &Path, label: &str, ordinal: u64) {
    let recovered = crate::WorthStoreRecovery::recover(healthy_recovery_request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = recovered else {
        panic!("{label}#{ordinal}: subsequent clean recovery must succeed, got {recovered:?}");
    };
    drop(handoff);
}

fn recovery_limits() -> PhysicalRecoveryLimits {
    PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: 64 << 20,
        manifest_entries: 4096,
        wal_segments: 64,
        wal_frames: 4096,
        wal_bytes: 64 << 20,
        redo_targets: 4096,
        redo_bytes: 64 << 20,
        distinct_pages_and_extents: 4096,
        operation_bindings: 4096,
        staging_bytes: 64 << 20,
        recovery_memory_bytes: RECOVERY_MEMORY_BYTES,
        dirty_frames: 4096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4096,
        cleanup_bytes: 64 << 20,
        observation_bytes: 64 << 20,
    })
    .unwrap()
}

fn fail_before() -> MediaFaultDirective {
    MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    }
}

fn faulted_identified_request(
    root: &Path,
    role: MediaOperationRole,
    nth: u64,
) -> (
    PhysicalRecoveryOpenRequest,
    CertificationMediaFaultActivation,
) {
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let fault_authority = media.fault_schedule_authority();
    let activation = fault_authority.one_shot_activation();
    let rule = fault_authority
        .rule(role, 1, fail_before())
        .for_nth_identified_operation_after_activation(
            activation.clone(),
            NonZeroU64::new(nth).unwrap(),
        );
    let schedule = fault_authority.schedule(vec![rule]).unwrap();
    (
        try_request_with_schedule(root, schedule).expect("certification authority admits"),
        activation,
    )
}

fn try_faulted_raw_ordinal_request(
    root: &Path,
    role: MediaOperationRole,
    ordinal: u64,
) -> Result<PhysicalRecoveryOpenRequest, ()> {
    let media =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let fault_authority = media.fault_schedule_authority();
    let rule = fault_authority.rule(role, ordinal, fail_before());
    let schedule = fault_authority.schedule(vec![rule]).unwrap();
    try_request_with_schedule(root, schedule)
}

fn try_request_with_schedule(
    root: &Path,
    schedule: worth_store::physical_runtime::certification::MediaFaultSchedule,
) -> Result<PhysicalRecoveryOpenRequest, ()> {
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let limits = recovery_limits();
    let authority = PhysicalRecoveryPlatformAuthority::acquire_for_certification(
        root.to_owned(),
        configuration.clone(),
        limits,
        schedule,
    )
    .map_err(|_| ())?;
    let backend = authority.qualified_backend_profile().clone();
    Ok(PhysicalRecoveryOpenRequest::declare(
        root.to_owned(),
        configuration,
        backend,
        limits,
        authority,
    ))
}

fn healthy_recovery_request(root: &Path) -> PhysicalRecoveryOpenRequest {
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let limits = recovery_limits();
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.to_owned(), configuration.clone(), limits)
            .unwrap();
    let backend = authority.qualified_backend_profile().clone();
    PhysicalRecoveryOpenRequest::declare(root.to_owned(), configuration, backend, limits, authority)
}
