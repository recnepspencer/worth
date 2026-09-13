use super::*;

#[test]
fn relational_fork_destination_reservation_conflicts_and_releases_on_drop() {
    let (fixture, _owner, _expected) = setup();
    let held = fixture
        .reserve_relational_fork_target("relational-held-destination")
        .expect("the first Relational reservation is owner-issued");
    assert!(matches!(
        fixture.reserve_relational_fork_target("relational-held-destination"),
        Err(worth_relational::facade::branch::RelationalForkDenial::DuplicateTarget)
    ));
    drop(held);
    let released = fixture
        .reserve_relational_fork_target("relational-held-destination")
        .expect("dropping the owner reservation releases the exact target");
    drop(released);
}

#[test]
fn cancellation_before_effect_and_after_signal_effect_have_distinct_outcomes() {
    let (_fixture, owner, expected) = setup();
    let reservation_cancellation = RuntimeWorldCancellationSource::new();
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected.clone(),
        CompositePublicationIntent::with_signal(None),
        &reservation_cancellation.token(),
        None,
    )
    .expect("reservation completes before cancellation");
    reservation_cancellation.cancel();
    let mut context = ();
    let before = RuntimeWorldOwnerExecutionService::execute_with_signal(
        owner.as_ref(),
        prepared,
        &mut context,
        &reservation_cancellation.token(),
        |_| Ok(()),
    );
    assert!(matches!(
        before,
        OwnerExecutionOutcome::NoEffect(no_effect)
            if no_effect.cause() == crate::publication::NoEffectCause::CancelledBeforeEffect
    ));

    let prepared = prepare_signal(&owner, &expected, None);
    let runtime_cancellation = RuntimeWorldCancellationSource::new();
    let runtime_token = runtime_cancellation.token();
    let mut context = ();
    let after = RuntimeWorldOwnerExecutionService::execute_with_signal(
        owner.as_ref(),
        prepared,
        &mut context,
        &runtime_token,
        |_| {
            runtime_cancellation.cancel();
            Ok(())
        },
    );
    let retained = match after {
        OwnerExecutionOutcome::ProductUnpublished(retained) => retained,
        other => panic!("post-effect cancellation must retain effects: {other:?}"),
    };
    assert_eq!(
        retained.cause(),
        ProductUnpublishedCause::CancellationAfterEffect
    );
    assert_eq!(retained.owner_effect_count(), 1);
    drop(retained);
}

#[test]
fn stale_product_head_is_denied_before_the_first_owner_effect() {
    let (fixture, owner, expected) = setup_with_relational_source();
    let competing_ready = ready_relational_competitor(
        &fixture,
        owner.as_ref(),
        &expected,
        "stale-before-effect-competitor",
    );
    let prepared = prepare_signal(&owner, &expected, None);
    let winner = publish_ready_competing_head(owner.as_ref(), competing_ready, &expected);
    let outcome = execute_with_empty_signal(&owner, prepared);
    let no_effect = match outcome {
        OwnerExecutionOutcome::NoEffect(no_effect) => no_effect,
        other => panic!("stale product head must deny before effect: {other:?}"),
    };
    assert_eq!(
        no_effect.cause(),
        crate::publication::NoEffectCause::StaleExpectedProductHead
    );
    assert_eq!(
        no_effect.observed_head().unwrap().selected_commit(),
        winner.selected_commit()
    );
    assert!(
        crate::basis::validate_current(
            &owner.state.relational.basis_port(),
            &owner.state.signal.basis_port(),
            &owner.state.bridge,
            winner.basis().relational_basis(),
            winner.basis().signal_basis(),
            winner.basis().correspondence_basis(),
        )
        .is_ok(),
        "the stale expected head is denied before any owner contact: the winner's exact component bases are still current"
    );
    assert_eq!(owner.recovery_record_count(), 0);
}

/// The Signal sibling denies its advance after the Relational effect has
/// already settled. The record names the exact settled Relational evidence and
/// an untouched Signal owner; nothing is reclassified as no-effect.
#[test]
fn missing_signal_sibling_after_relational_movement_retains_exact_progress() {
    let (fixture, owner, expected) = setup();
    let prepared = prepare_both_owners(&fixture, &owner, &expected, "missing-signal-sibling");
    let cancellation = RuntimeWorldCancellationSource::new();
    let mut context = ();
    let record = retained(RuntimeWorldOwnerExecutionService::execute_with_signal(
        owner.as_ref(),
        prepared,
        &mut context,
        &cancellation.token(),
        |_| {
            Err(worth_signal::facade::SignalError::invalid_input(
                "the Signal sibling is absent from this composition",
            ))
        },
    ));
    assert_retains_only_the_relational_effect(&record, ProductUnpublishedCause::SiblingOwnerDenied);
    assert_eq!(
        record.next_actions(),
        [
            ProductUnpublishedNextAction::StartFreshCompositePublication,
            ProductUnpublishedNextAction::ReleaseObligations,
            ProductUnpublishedNextAction::Inspect,
        ],
        "a minted record whose Relational publication evidence is settled advertises a fresh composite attempt, in the authority's order"
    );
    assert!(
        record.successor_basis().is_some(),
        "the retained record names the successor basis the settled effect reached"
    );
    drop(record);
}

#[test]
fn actual_owner_loss_during_successor_retention_remains_owner_lost() {
    let (fixture, owner, expected) = setup();
    let settlement = settled(execute_with_empty_signal(
        &owner,
        prepare_signal(&owner, &expected, None),
    ));
    let successor = settlement.successor_basis().unwrap().clone();
    drop(fixture);
    let retained = settlement
        .ready(successor)
        .expect_err("the dropped component owner cannot issue a new successor pin");
    assert_eq!(retained.cause(), ProductUnpublishedCause::OwnerLost);
    assert!(retained
        .next_actions()
        .contains(&ProductUnpublishedNextAction::CloseOwner));
    assert_eq!(retained.owner_effect_count(), 1);
    let handle = retained.recovery_handle();
    drop(retained);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.recovery_record_count(), 0);
}
