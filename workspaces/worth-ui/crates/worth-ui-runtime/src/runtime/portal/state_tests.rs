use super::{
    test_support::{
        idempotency, open_request, portal, presented_geometry, semantic_surface, state,
        viewport_bounds,
    },
    UiPortalDismissalCause, UiPortalLifecyclePosture, UiPortalServiceDisposition,
    UiPortalServiceRequest, UiPortalServiceTransitionDenial,
};

#[test]
fn preparation_changes_no_portal_truth_or_counters() {
    let state = state();
    let portal = portal(11, 21);
    let prepared = state
        .prepare(open_request(portal, 31))
        .expect("revision remains available");

    assert_eq!(prepared.staged_posture(), UiPortalLifecyclePosture::Open);
    assert_eq!(state.posture(portal), UiPortalLifecyclePosture::Closed);
    assert_eq!(state.active_count(), 0);
    assert_eq!(state.admitted_requests(), 0);
    assert_eq!(state.idempotent_requests(), 0);
    assert_eq!(state.revision(), 0);
}

#[test]
fn published_open_and_close_commit_physical_portal_truth() {
    let mut state = state();
    let portal = portal(41, 51);
    let open = state
        .prepare(open_request(portal, 61))
        .expect("open prepares");
    let opened = state.commit_published(open).expect("open remains current");
    let surface = state
        .semantic_surface_for_test(portal)
        .expect("visible portal retains its semantic surface");

    assert_eq!(opened.disposition(), UiPortalServiceDisposition::Opened);
    assert_eq!(opened.posture(), UiPortalLifecyclePosture::Visible);
    assert_eq!(state.active_count(), 1);

    let close = state
        .prepare(UiPortalServiceRequest::close(
            portal,
            idempotency(62),
            UiPortalDismissalCause::Escape,
            surface,
        ))
        .expect("close prepares");
    let closed = state
        .commit_published(close)
        .expect("close remains current");

    assert_eq!(closed.disposition(), UiPortalServiceDisposition::Closing);
    assert_eq!(closed.posture(), UiPortalLifecyclePosture::Closed);
    assert_eq!(state.active_count(), 0);
    assert_eq!(state.admitted_requests(), 2);
}

#[test]
fn exit_retention_keeps_closing_projection_until_exact_terminal_publication() {
    let mut state = state();
    let portal = portal(43, 53);
    let open = state.prepare(open_request(portal, 63)).unwrap();
    state.commit_published(open).unwrap();
    let surface = state
        .semantic_surface_for_test(portal)
        .expect("closing portal retains its semantic surface");
    let placement = state
        .placement(portal)
        .expect("visible portal has placement");
    let original_ordinal = state.stack_snapshot().rows()[0].ordinal();
    let close = state
        .prepare(UiPortalServiceRequest::close(
            portal,
            idempotency(64),
            UiPortalDismissalCause::Escape,
            surface,
        ))
        .unwrap();
    let closing_projection = state.mounted_projection_inputs(&close, true);
    assert_eq!(closing_projection.len(), 1);
    assert_eq!(
        closing_projection[0].lifecycle(),
        UiPortalLifecyclePosture::Closing
    );
    let (_, retention) = state
        .commit_published_with_exit_retention(close, true)
        .unwrap();
    let retention = retention.expect("retained close issues exact receipt");

    assert_eq!(state.posture(portal), UiPortalLifecyclePosture::Closing);
    assert_eq!(state.placement(portal), Some(placement));
    assert_eq!(state.active_count(), 1);
    let closing_snapshot = state.stack_snapshot();
    assert_eq!(closing_snapshot.rows().len(), 1);
    assert_eq!(closing_snapshot.rows()[0].portal(), portal);
    assert_eq!(closing_snapshot.rows()[0].ordinal(), original_ordinal);
    let mismatched = super::UiPortalExitRetentionReceipt::new(
        portal,
        retention.revision() + 1,
        retention.causal_lineage(),
    );
    assert!(matches!(
        state.prepare_exit_terminal(mismatched, idempotency(65)),
        Err(super::UiPortalExitTerminalDenial::RetentionMismatch)
    ));

    let terminal = state
        .prepare_exit_terminal(retention, idempotency(66))
        .expect("exact retention authorizes terminal close");
    assert!(state.mounted_projection_inputs(&terminal, false).is_empty());
    state
        .commit_published_with_exit_retention(terminal, false)
        .unwrap();
    assert_eq!(state.posture(portal), UiPortalLifecyclePosture::Closed);
    assert_eq!(state.placement(portal), None);
    assert_eq!(state.active_count(), 0);
    assert!(matches!(
        state.prepare_exit_terminal(retention, idempotency(67)),
        Err(super::UiPortalExitTerminalDenial::RetentionMismatch)
    ));
}

#[test]
fn shutdown_terminalizes_visible_portals() {
    let mut state = state();
    let visible = portal(42, 52);
    let visible_open = state.prepare(open_request(visible, 63)).unwrap();
    state.commit_published(visible_open).unwrap();

    let report = state.shutdown();

    assert_eq!(report.closed_records(), 1);
    assert_eq!(report.abandoned_indeterminate_records(), 0);
    assert_eq!(report.final_active_records(), 0);
}

#[test]
fn exact_settled_requests_are_idempotent_only_after_commit() {
    let mut state = state();
    let portal = portal(71, 81);
    let request = open_request(portal, 91);
    let first = state.prepare(request).expect("first request prepares");
    state
        .commit_published(first)
        .expect("first request commits");
    let duplicate = state.prepare(request).expect("duplicate prepares");

    assert_eq!(
        duplicate.staged_posture(),
        UiPortalLifecyclePosture::Visible
    );
    assert_eq!(state.idempotent_requests(), 0);
    let receipt = state
        .commit_published(duplicate)
        .expect("duplicate remains current");
    assert_eq!(
        receipt.disposition(),
        UiPortalServiceDisposition::Idempotent
    );
    assert_eq!(state.idempotent_requests(), 1);
    assert_eq!(state.admitted_requests(), 2);
}

#[test]
fn reused_idempotency_with_changed_dismissal_cause_is_not_an_exact_duplicate() {
    let mut state = state();
    let portal = portal(92, 93);
    let idempotency = idempotency(94);
    let open = state.prepare(open_request(portal, 95)).unwrap();
    state.commit_published(open).unwrap();
    let surface = state
        .semantic_surface_for_test(portal)
        .expect("live portal retains its semantic surface");
    let first = state
        .prepare(UiPortalServiceRequest::close(
            portal,
            idempotency,
            UiPortalDismissalCause::Escape,
            surface,
        ))
        .expect("first close prepares");
    state
        .commit_published_with_exit_retention(first, true)
        .expect("first close remains current");

    let changed = state
        .prepare(UiPortalServiceRequest::close(
            portal,
            idempotency,
            UiPortalDismissalCause::OutsidePress,
            surface,
        ))
        .expect("changed close prepares as fresh work");
    assert_eq!(changed.staged_posture(), UiPortalLifecyclePosture::Closing);
    let receipt = state
        .commit_published(changed)
        .expect("changed close settles");
    assert_eq!(receipt.disposition(), UiPortalServiceDisposition::Closing);
    assert_eq!(state.idempotent_requests(), 0);
}

#[test]
fn a_stale_prepared_transition_cannot_overwrite_newer_truth() {
    let mut state = state();
    let original = portal(101, 111);
    let replacement = portal(101, 112);
    let stale = state
        .prepare(open_request(original, 121))
        .expect("first transition prepares");
    let current = state
        .prepare(open_request(replacement, 122))
        .expect("parallel transition prepares from the same revision");
    state
        .commit_published(current)
        .expect("one transition commits");

    assert_eq!(
        state.commit_published(stale),
        Err(UiPortalServiceTransitionDenial::StalePlan)
    );
    assert_eq!(state.posture(original), UiPortalLifecyclePosture::Closed);
    assert_eq!(
        state.posture(replacement),
        UiPortalLifecyclePosture::Visible
    );
    assert_eq!(state.active_count(), 1);
    assert_eq!(state.admitted_requests(), 1);
}

#[test]
fn nested_modal_portal_carries_parent_depth_and_surface_shielding() {
    let mut state = state();
    let parent = portal(161, 171);
    let child = portal(162, 172);
    let parent_transition = state
        .prepare(open_request(parent, 181))
        .expect("parent opens");
    state
        .commit_published(parent_transition)
        .expect("parent publishes");
    let geometry = presented_geometry(2);
    let child_transition = state
        .prepare(UiPortalServiceRequest::open_nested(
            child,
            idempotency(182),
            geometry,
            viewport_bounds(geometry),
            semantic_surface(),
            parent,
            super::UiPortalInputShielding::ModalSurface,
        ))
        .expect("child resolves through its committed parent");
    let placement = child_transition.placement().expect("open has placement");

    assert_eq!(placement.layer().parent(), Some(parent));
    assert_eq!(placement.layer().depth(), 1);
    assert_eq!(
        placement.shielding(),
        super::UiPortalInputShielding::ModalSurface
    );
}

#[test]
fn nested_portal_rejects_an_uncommitted_parent_before_truth_changes() {
    let state = state();
    let parent = portal(191, 201);
    let child = portal(192, 202);

    let geometry = presented_geometry(3);
    assert!(matches!(
        state.prepare(UiPortalServiceRequest::open_nested(
            child,
            idempotency(211),
            geometry,
            viewport_bounds(geometry),
            semantic_surface(),
            parent,
            super::UiPortalInputShielding::ContentBounds,
        )),
        Err(UiPortalServiceTransitionDenial::Placement(
            super::UiPortalPlacementDenial::UnknownParent
        ))
    ));
    assert_eq!(state.active_count(), 0);
    assert_eq!(state.revision(), 0);
}

#[test]
fn stack_snapshot_uses_minted_total_order_not_identity_or_depth() {
    let mut state = state();
    let first = portal(301, 401);
    let second = portal(201, 402);
    let first_request = open_request(first, 501);
    let open_first = state.prepare(first_request).unwrap();
    state.commit_published(open_first).unwrap();
    let open_second = state.prepare(open_request(second, 502)).unwrap();
    state.commit_published(open_second).unwrap();

    let snapshot = state.stack_snapshot();
    assert_eq!(
        snapshot
            .rows()
            .iter()
            .map(|row| row.portal())
            .collect::<Vec<_>>(),
        [first, second]
    );
    assert!(snapshot.rows()[0].ordinal().value() < snapshot.rows()[1].ordinal().value());

    let duplicate = state.prepare(first_request).unwrap();
    state.commit_published(duplicate).unwrap();
    let retained = state.stack_snapshot();
    assert_eq!(retained.rows()[0].ordinal(), snapshot.rows()[0].ordinal());
}

#[test]
fn sibling_open_mints_a_new_ordinal_and_exhaustion_denies_before_effects() {
    let mut state = state();
    let original = portal(601, 701);
    let replacement = portal(601, 702);
    let open = state.prepare(open_request(original, 801)).unwrap();
    state.commit_published(open).unwrap();
    let original_ordinal = state.stack_snapshot().rows()[0].ordinal();

    let replace = state.prepare(open_request(replacement, 802)).unwrap();
    state.commit_published(replace).unwrap();
    let replacement_ordinal = state
        .stack_snapshot()
        .rows()
        .iter()
        .find(|row| row.portal() == replacement)
        .expect("replacement remains in the total stack")
        .ordinal();
    assert!(replacement_ordinal > original_ordinal);

    state.force_next_stack_ordinal(u64::MAX);
    let final_portal = portal(603, 703);
    let final_open = state
        .prepare(open_request(final_portal, 803))
        .expect("the maximum ordinal is still issuable");
    state
        .commit_published(final_open)
        .expect("the maximum ordinal remains publishable");
    assert_eq!(
        state
            .stack_snapshot()
            .rows()
            .last()
            .unwrap()
            .ordinal()
            .value(),
        u64::MAX
    );
    let before_revision = state.revision();
    let before_snapshot = state.stack_snapshot();
    let before_cursor = state.stack_ordinal_issuer.next();
    let before_active = state.active_count();
    let before_admitted = state.admitted_requests();
    assert!(matches!(
        state.prepare(open_request(portal(604, 704), 804)),
        Err(UiPortalServiceTransitionDenial::StackOrdinalExhausted)
    ));
    assert_eq!(state.revision(), before_revision);
    assert_eq!(state.stack_snapshot(), before_snapshot);
    assert_eq!(state.stack_ordinal_issuer.next(), before_cursor);
    assert_eq!(state.active_count(), before_active);
    assert_eq!(state.admitted_requests(), before_admitted);
}
