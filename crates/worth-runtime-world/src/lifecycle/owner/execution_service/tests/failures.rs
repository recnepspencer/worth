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
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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
        execution,
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
        execution,
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
            &owner.state.relational_services().basis_port(),
            &owner.state.signal.basis_port(),
            owner.state.bridge.as_ref().expect("serving World owns correspondence"),
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
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let prepared = prepare_both_owners(
        &execution,
        &fixture,
        &owner,
        &expected,
        "missing-signal-sibling",
    );
    let cancellation = RuntimeWorldCancellationSource::new();
    let mut context = ();
    let record = retained(RuntimeWorldOwnerExecutionService::execute_with_signal(
        owner.as_ref(),
        execution,
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

#[test]
fn retired_branch_second_frame_busy_returns_resume_commit_and_treats_fence_as_held() {
    use super::durability_unresolved::{
        setup_history_chain, ScriptedHistoryAnswer as Ans, ScriptedHistoryMedium,
    };
    use crate::durability::WorldDurableMediumBinding;
    use crate::history::HistoryReclamationDenial;

    // 4 unowned commits (chain[1..=4]) above protected root chain[0]; drop tip_guard so
    // the whole retired branch tail [4, 3, 2, 1] is eligible leaf-to-root.
    let (_fx1, owner, chain, _rg1, tip_guard1) = setup_history_chain(4);
    drop(tip_guard1);
    let policy = owner.state.execution.request_policy();
    let serial = worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let exec = worth_execution::ExecutionRequest::serial(&serial);
    let medium = ScriptedHistoryMedium::new(vec![Ans::Commit, Ans::Busy]);
    let binding = Arc::new(WorldDurableMediumBinding::new(medium.clone(), &exec).unwrap());
    let pending = owner
        .state
        .history
        .begin_durable_release_retired_branch_history(chain[4].identity())
        .unwrap();
    let err = owner
        .drive_durable_history_retirement_with_chunk_size(&binding, pending, 2)
        .unwrap_err();
    let resume_from = match err {
        HistoryReclamationDenial::PartialPublication {
            reclaimed,
            resume_from: Some(next),
            indeterminate: false,
        } => {
            assert_eq!(
                reclaimed,
                vec![chain[4].identity().clone(), chain[3].identity().clone()]
            );
            assert_eq!(next, chain[2].identity().clone());
            next
        }
        other => panic!("expected PartialPublication with resume_from=c2, got {other:?}"),
    };
    assert!(owner.state.history.lookup(chain[4].identity()).is_none());
    assert!(owner.state.history.lookup(chain[3].identity()).is_none());
    // Retrying from the exact resume_from reclaims the remaining suffix [c2, c1] once.
    medium.answers.lock().unwrap().push_back(Ans::Commit);
    let retry = owner
        .state
        .history
        .begin_durable_release_retired_branch_history(&resume_from)
        .unwrap();
    let (reclaimed_suffix, held) = owner
        .drive_durable_history_retirement_with_chunk_size(&binding, retry, 2)
        .unwrap();
    assert!(held.is_none());
    assert_eq!(
        reclaimed_suffix,
        vec![chain[2].identity().clone(), chain[1].identity().clone()]
    );
    assert!(owner.state.history.lookup(chain[2].identity()).is_none());
    assert!(owner.state.history.lookup(chain[1].identity()).is_none());
    assert!(owner.state.history.lookup(chain[0].identity()).is_some());

    // Case 2: second frame Unresolved leaves [c2, c1] fenced; walking to a fenced commit
    // MUST report held=Some(c2) rather than completion (held=None).
    let (_fx2, owner2, chain2, _rg2, tip_guard2) = setup_history_chain(4);
    drop(tip_guard2);
    let medium2 = ScriptedHistoryMedium::new(vec![Ans::Commit, Ans::Unresolved]);
    let binding2 = Arc::new(WorldDurableMediumBinding::new(medium2, &exec).unwrap());
    let pending2 = owner2
        .state
        .history
        .begin_durable_release_retired_branch_history(chain2[4].identity())
        .unwrap();
    let err2 = owner2
        .drive_durable_history_retirement_with_chunk_size(&binding2, pending2, 2)
        .unwrap_err();
    let fenced_head = match err2 {
        HistoryReclamationDenial::PartialPublication {
            reclaimed,
            resume_from: Some(next),
            indeterminate: true,
        } => {
            assert_eq!(
                reclaimed,
                vec![chain2[4].identity().clone(), chain2[3].identity().clone()]
            );
            assert_eq!(next, chain2[2].identity().clone());
            next
        }
        other => panic!("expected indeterminate PartialPublication at c2, got {other:?}"),
    };
    let hit_fence = owner2
        .state
        .history
        .begin_durable_release_retired_branch_history(&fenced_head)
        .unwrap();
    let (reclaimed_fenced, held_fenced) = owner2
        .drive_durable_history_retirement_with_chunk_size(&binding2, hit_fence, 2)
        .unwrap();
    assert!(reclaimed_fenced.is_empty());
    assert_eq!(
        held_fenced,
        Some(chain2[2].identity().clone()),
        "a walk reaching a fenced commit must report held rather than finished"
    );
}

#[test]
fn signal_only_horizon_creation_rejects_mismatched_native_predecessor() {
    use crate::durability::WorldReadmissionDenial as Denial;
    use worth_relational::facade::branch::RecordedRelationalComponentReference as Recorded;

    let (_fixture, owner, expected) = setup();
    let policy = owner.state.execution.request_policy();
    let serial = worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let exec = worth_execution::ExecutionRequest::serial(&serial);
    let before = owner
        .state
        .relational_services()
        .basis_port()
        .export_recorded_retained_component(&exec, expected.basis().relational_basis())
        .unwrap();
    assert!(
        crate::lifecycle::builder::require_retained_native_predecessor_for_test(&before, &before)
            .is_ok()
    );
    let mut wrong_branch = serde_json::to_value(&before).unwrap();
    wrong_branch["descriptor"]["branch_id"] = serde_json::json!("forged-native-branch");
    let wrong_branch: Recorded = serde_json::from_value(wrong_branch).unwrap();
    assert!(matches!(
        crate::lifecycle::builder::require_retained_native_predecessor_for_test(
            &wrong_branch,
            &before,
        ),
        Err(Denial::CreationEvidenceMismatch)
    ));
    let mut wrong_truth = serde_json::to_value(&before).unwrap();
    let truth = wrong_truth["descriptor"]["truth_version"].as_u64().unwrap();
    wrong_truth["descriptor"]["truth_version"] = serde_json::json!(truth + 1);
    let wrong_truth: Recorded = serde_json::from_value(wrong_truth).unwrap();
    assert!(matches!(
        crate::lifecycle::builder::require_retained_native_predecessor_for_test(
            &wrong_truth,
            &before,
        ),
        Err(Denial::CreationEvidenceMismatch)
    ));
}
