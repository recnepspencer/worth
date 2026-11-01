//! Guard only the actual memory cutover; user preparation and Drop stay outside.
use super::*;
use std::sync::{atomic::Ordering, Arc};
use worth_relational::facade::mvcc::{
    RelationalInterruptionBoundary, RelationalOperationControl, RelationalOperationInterruption,
};
mod payload;
use payload::{register, Probe};

#[test]
fn real_companion_preparation_precedes_handoff_and_retired_payload_drops_after_it() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let probe = Arc::new(Probe::default());
    let _registration = register(&runtime, Arc::clone(&probe));
    let candidate = prepare(execution, &runtime, &source);
    let scoped = runtime
        .publication_port()
        .with_prepared_memory_cutover(execution, candidate, |cutover| {
            assert_eq!(probe.prepares.load(Ordering::SeqCst), 1);
            let _active = probe.enter();
            cutover.perform();
            assert!(
                probe.events().is_empty(),
                "an owning payload died under the guard"
            );
            Err::<(), _>("caller status cannot replace actual performed custody")
        })
        .unwrap();
    let (handoff, outcome) = scoped.into_parts();
    assert!(handoff.is_err());
    assert!(
        probe.events().contains(&(0, false)),
        "the original retired image must actually drop"
    );
    assert!(probe.events().iter().all(|(_, active)| !active));
    let RelationalPublicationOutcome::Performed(performed) = outcome.unwrap() else {
        panic!("the actual cutover result must survive the caller's status");
    };
    let result = runtime
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap();
    assert_eq!(changed_entities(&result).len(), 1);
    release_test_commit_snapshot(&runtime, &result);
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn refused_cutover_keeps_prepared_user_payload_alive_until_handoff_ends() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let probe = Arc::new(Probe::default());
    let _registration = register(&runtime, Arc::clone(&probe));
    let control = RelationalOperationControl::uninterrupted().with_injected_interruption_for_test(
        RelationalInterruptionBoundary::BeforeCriticalSection,
        RelationalOperationInterruption::Cancelled,
        std::num::NonZeroUsize::MIN,
    );
    let candidate = prepare_with_control(execution, &runtime, &source, control);
    let (_, outcome) = runtime
        .publication_port()
        .with_prepared_memory_cutover(execution, candidate, |cutover| {
            let _active = probe.enter();
            cutover.perform();
            assert!(
                probe.events().is_empty(),
                "refusal destroyed a payload under the guard"
            );
        })
        .unwrap()
        .into_parts();
    assert!(matches!(
        outcome,
        Some(RelationalPublicationOutcome::Interrupted(_))
    ));
    assert!(
        probe.events().contains(&(1, false)),
        "the actual refused successor must be released"
    );
    assert!(probe.events().iter().all(|(_, active)| !active));
    assert_eq!(
        runtime
            .admit_branch_basis(source.identity())
            .unwrap()
            .descriptor(),
        source.descriptor()
    );
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn unused_cutover_token_releases_original_preflight_without_moving_the_head() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let probe = Arc::new(Probe::default());
    let _registration = register(&runtime, Arc::clone(&probe));
    let candidate = prepare(execution, &runtime, &source);
    let (caller, outcome) = runtime
        .publication_port()
        .with_prepared_memory_cutover(execution, candidate, |_cutover| {
            let _active = probe.enter();
            assert!(probe.events().is_empty());
            7
        })
        .unwrap()
        .into_parts();
    assert_eq!(caller, 7);
    assert!(outcome.is_none());
    assert!(probe.events().contains(&(1, false)));
    assert_eq!(
        runtime
            .admit_branch_basis(source.identity())
            .unwrap()
            .descriptor(),
        source.descriptor()
    );
    // Original source and companion remain usable after skipped work releases.
    let result = perform(execution, &runtime, prepare(execution, &runtime, &source));
    assert_eq!(changed_entities(&result).len(), 1);
    release_test_commit_snapshot(&runtime, &result);
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn unwinding_handoff_drops_user_payload_after_the_guard_unwinds() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let probe = Arc::new(Probe::default());
    let _registration = register(&runtime, Arc::clone(&probe));
    let candidate = prepare(execution, &runtime, &source);
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = runtime.publication_port().with_prepared_memory_cutover(
            execution,
            candidate,
            |_cutover| {
                let _active = probe.enter();
                panic!("stop after actual preflight, before cutover");
            },
        );
    }));
    assert!(caught.is_err());
    assert!(probe.events().contains(&(1, false)));
    assert!(probe.events().iter().all(|(_, active)| !active));
    assert_eq!(
        runtime
            .admit_branch_basis(source.identity())
            .unwrap()
            .descriptor(),
        source.descriptor()
    );
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn attached_medium_refuses_scoped_cutover_before_callback_or_medium_access() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(fixture().build_registry())
        .build_durable(Box::new(super::super::unaccessed_medium::UnaccessedMedium))
        .unwrap();
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let candidate = prepare(execution, &runtime, &source);
    let result =
        runtime
            .publication_port()
            .with_prepared_memory_cutover(execution, candidate, |_cutover| {
                panic!("unsupported durability entered the guard callback")
            });
    assert!(matches!(
        result,
        Err(RelationalPublicationOutcome::Denied(
            RelationalPublicationDenial::ScopedCutoverDurabilityUnsupported
        ))
    ));
    assert_eq!(
        runtime
            .admit_branch_basis(source.identity())
            .unwrap()
            .descriptor(),
        source.descriptor()
    );
}
