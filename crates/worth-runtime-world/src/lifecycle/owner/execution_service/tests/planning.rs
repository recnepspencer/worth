use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::*;

#[test]
fn relational_only_executes_the_canonical_relational_owner_path() {
    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(&fixture, &owner, &expected, "relational-only");
    let settlement = settled(execute_without_signal(&owner, prepared));
    assert_eq!(
        settlement.progress().relational_posture(),
        RelationalAttemptProgressPosture::Settled
    );
    assert_eq!(
        settlement.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
    assert_ne!(
        settlement.successor_basis().unwrap().relational_basis(),
        expected.basis().relational_basis()
    );
    let successor = settlement.successor_basis().cloned().unwrap();
    settlement
        .ready(successor)
        .expect("settled Relational evidence forms a ready product publication");
}

#[test]
fn signal_only_uses_the_real_mutation_owner_without_relational_contact() {
    let (_fixture, owner, expected) = setup();
    let prepared = prepare_signal(&owner, &expected, None);
    let settlement = settled(execute_with_empty_signal(&owner, prepared));
    assert_eq!(
        settlement.progress().relational_posture(),
        RelationalAttemptProgressPosture::Untouched
    );
    assert_eq!(
        settlement.progress().signal_posture(),
        SignalAttemptProgressPosture::Performed
    );
    assert_ne!(
        settlement
            .successor_basis()
            .unwrap()
            .signal_basis()
            .admission_identity(),
        expected.basis().signal_basis().admission_identity()
    );
    let successor = settlement.successor_basis().cloned().unwrap();
    settlement
        .ready(successor)
        .expect("settled Signal evidence forms a ready product publication");
}

#[test]
fn both_changed_settles_relational_before_signal_and_preserves_both_bases() {
    let (fixture, owner, expected) = setup();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected.clone(),
        CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate("both-changed"),
            ),
        &cancellation.token(),
        None,
    )
    .expect("the exact observed head admits both owner legs");
    let relational_branch = expected.basis().relational_basis().identity().clone();
    let expected_relational = expected.basis().relational_basis().clone();
    let relational_was_observed = Arc::new(AtomicBool::new(false));
    let observed_for_callback = Arc::clone(&relational_was_observed);
    let owner_for_callback = Arc::clone(&owner);
    let mut context = ();
    let settlement = settled(RuntimeWorldOwnerExecutionService::execute_with_signal(
        owner.as_ref(),
        prepared,
        &mut context,
        &cancellation.token(),
        move |_| {
            let (_, observed) = owner_for_callback
                .state
                .relational
                .basis_port()
                .observe_branch(&relational_branch)
                .map_err(|_| {
                    worth_signal::facade::SignalError::invalid_input(
                        "Relational successor observation was unavailable",
                    )
                })?;
            if observed == expected_relational {
                return Err(worth_signal::facade::SignalError::invalid_input(
                    "Signal owner ran before the Relational successor",
                ));
            }
            observed_for_callback.store(true, Ordering::Release);
            Ok(())
        },
    ));
    assert!(
        relational_was_observed.load(Ordering::Acquire),
        "the real Signal callback must observe the Relational successor"
    );
    assert_eq!(
        settlement.progress().relational_posture(),
        RelationalAttemptProgressPosture::Settled
    );
    assert_eq!(
        settlement.progress().signal_posture(),
        SignalAttemptProgressPosture::Performed
    );
    let successor = settlement.successor_basis().cloned().unwrap();
    assert_ne!(
        successor.relational_basis(),
        expected.basis().relational_basis()
    );
    assert_ne!(
        successor.signal_basis().admission_identity(),
        expected.basis().signal_basis().admission_identity()
    );
    settlement
        .ready(successor.clone())
        .expect("the ordered owner results form one exact successor");
}

#[test]
fn a_relational_publication_never_contacts_the_signal_owner() {
    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(&fixture, &owner, &expected, "reuse-signal");
    let settlement = settled(execute_without_signal(&owner, prepared));
    assert_eq!(
        settlement.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
    assert_eq!(
        settlement.progress().owner_effect_count(),
        1,
        "only the requested Relational owner moved"
    );
}
