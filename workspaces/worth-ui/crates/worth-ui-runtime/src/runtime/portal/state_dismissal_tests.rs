use super::{
    test_support::{
        idempotency, open_request, portal, presented_geometry, semantic_surface, state,
        viewport_bounds,
    },
    UiPortalDismissalCause, UiPortalDismissalIgnoreReason, UiPortalDismissalPreparation,
    UiPortalDismissalTrigger, UiPortalInputShielding, UiPortalLifecyclePosture,
    UiPortalServiceRequest,
};

#[test]
fn escape_and_anchor_loss_dismiss_nested_portals_in_topmost_order() {
    let mut state = state();
    let parent = portal(221, 231);
    let child = portal(222, 232);
    let parent_open = state.prepare(open_request(parent, 241)).unwrap();
    state.commit_published(parent_open).unwrap();
    let geometry = presented_geometry(4);
    let child_open = state
        .prepare(UiPortalServiceRequest::open_nested(
            child,
            idempotency(242),
            geometry,
            viewport_bounds(geometry),
            semantic_surface(),
            parent,
            UiPortalInputShielding::ModalSurface,
        ))
        .unwrap();
    state.commit_published(child_open).unwrap();

    let UiPortalDismissalPreparation::Prepared(dismiss_child) = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::Escape {
                semantic_surface: state.semantic_surface_for_test(child).unwrap(),
            },
            None,
            idempotency(243),
        )
        .unwrap()
    else {
        panic!("Escape must dismiss the topmost nested portal")
    };
    assert_eq!(dismiss_child.portal(), child);
    assert!(dismiss_child.input_shielded());
    state
        .commit_published(dismiss_child.into_transition())
        .unwrap();
    let closed = state
        .last_closed()
        .expect("the owner retains its latest close cause");
    assert_eq!(closed.portal(), child);
    assert_eq!(closed.cause(), UiPortalDismissalCause::Escape);

    let UiPortalDismissalPreparation::Prepared(dismiss_parent) = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::AnchorLoss(parent),
            None,
            idempotency(244),
        )
        .unwrap()
    else {
        panic!("anchor loss must dismiss its remaining portal")
    };
    assert_eq!(dismiss_parent.portal(), parent);
    state
        .commit_published(dismiss_parent.into_transition())
        .unwrap();
    assert_eq!(state.active_count(), 0);
}

#[test]
fn anchor_loss_targets_the_exact_anchor_and_closes_its_descendant_chain() {
    let mut state = state();
    let parent = portal(223, 233);
    let child = portal(224, 234);
    let parent_open = state.prepare(open_request(parent, 245)).unwrap();
    state.commit_published(parent_open).unwrap();
    let geometry = presented_geometry(5);
    let child_open = state
        .prepare(UiPortalServiceRequest::open_nested(
            child,
            idempotency(246),
            geometry,
            viewport_bounds(geometry),
            semantic_surface(),
            parent,
            UiPortalInputShielding::ModalSurface,
        ))
        .unwrap();
    state.commit_published(child_open).unwrap();

    let UiPortalDismissalPreparation::Prepared(dismiss_parent) = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::AnchorLoss(parent),
            None,
            idempotency(247),
        )
        .unwrap()
    else {
        panic!("anchor loss must prepare the exact lost anchor")
    };
    assert_eq!(dismiss_parent.portal(), parent);
    state
        .commit_published(dismiss_parent.into_transition())
        .unwrap();

    assert_eq!(state.active_count(), 0);
    let closed = state.last_closed().expect("anchor closure is inspected");
    assert_eq!(closed.portal(), parent);
    assert_eq!(closed.cause(), UiPortalDismissalCause::AnchorLoss);
}

#[test]
fn non_interaction_dismissal_causes_remain_typed_and_inspectable() {
    for (offset, cause) in [
        UiPortalDismissalCause::OwnerLoss,
        UiPortalDismissalCause::WindowFocusPolicy,
    ]
    .into_iter()
    .enumerate()
    {
        let mut state = state();
        let identity = portal(230 + offset as u64, 240 + offset as u64);
        let open = state
            .prepare(open_request(identity, 250 + offset as u64))
            .unwrap();
        state.commit_published(open).unwrap();
        let surface = state
            .semantic_surface_for_test(identity)
            .expect("live portal retains its semantic surface");
        let close = state
            .prepare(UiPortalServiceRequest::close(
                identity,
                idempotency(260 + offset as u64),
                cause,
                surface,
            ))
            .unwrap();
        state.commit_published(close).unwrap();
        assert_eq!(state.last_closed().unwrap().cause(), cause);
    }
}

#[test]
fn explicit_parent_close_atomically_closes_its_descendant_chain() {
    let mut state = state();
    let parent = portal(225, 235);
    let child = portal(226, 236);
    let parent_open = state.prepare(open_request(parent, 245)).unwrap();
    state.commit_published(parent_open).unwrap();
    let geometry = presented_geometry(5);
    let child_open = state
        .prepare(UiPortalServiceRequest::open_nested(
            child,
            idempotency(246),
            geometry,
            viewport_bounds(geometry),
            semantic_surface(),
            parent,
            UiPortalInputShielding::ContentBounds,
        ))
        .unwrap();
    state.commit_published(child_open).unwrap();
    let surface = state
        .semantic_surface_for_test(parent)
        .expect("parent retains its semantic surface");

    let close = state
        .prepare(UiPortalServiceRequest::close(
            parent,
            idempotency(247),
            UiPortalDismissalCause::ExplicitOwnerRequest,
            surface,
        ))
        .unwrap();
    assert!(state.mounted_projection_inputs(&close, false).is_empty());
    state.commit_published(close).unwrap();
    assert_eq!(state.posture(parent), UiPortalLifecyclePosture::Closed);
    assert_eq!(state.posture(child), UiPortalLifecyclePosture::Closed);
    assert_eq!(state.active_count(), 0);
}

#[test]
fn outside_press_respects_bounds_and_duplicate_dismissal_coalesces() {
    let mut state = state();
    let portal = portal(251, 261);
    let opened = state.prepare(open_request(portal, 271)).unwrap();
    let bounds = opened.placement().unwrap().bounds().components();
    state.commit_published(opened).unwrap();
    let surface = state.semantic_surface_for_test(portal).unwrap();
    let inside = [bounds[0] + 1.0, bounds[1] + 1.0].map(f32::to_bits);
    assert!(matches!(
        state
            .prepare_dismissal(
                UiPortalDismissalTrigger::OutsidePress {
                    semantic_surface: surface,
                    viewport_point_bits: inside
                },
                None,
                idempotency(272),
            )
            .unwrap(),
        UiPortalDismissalPreparation::Ignored(UiPortalDismissalIgnoreReason::InsideTopmostPortal)
    ));
    let sampled_bounds = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: bounds[0] + 100.0,
            y: bounds[1] + 100.0,
            width: bounds[2],
            height: bounds[3],
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .unwrap();
    let UiPortalDismissalPreparation::Prepared(dismissal) = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::OutsidePress {
                semantic_surface: surface,
                viewport_point_bits: inside,
            },
            Some(sampled_bounds),
            idempotency(273),
        )
        .unwrap()
    else {
        panic!("outside press must prepare dismissal")
    };
    state.commit_published(dismissal.into_transition()).unwrap();
    let revision = state.revision();
    assert!(matches!(
        state
            .prepare_dismissal(
                UiPortalDismissalTrigger::OutsidePress {
                    semantic_surface: surface,
                    viewport_point_bits: inside
                },
                None,
                idempotency(273),
            )
            .unwrap(),
        UiPortalDismissalPreparation::Ignored(UiPortalDismissalIgnoreReason::NoMatchingPortal)
    ));
    assert_eq!(state.revision(), revision);
}

#[test]
fn a_second_non_anchor_dismissal_republishes_a_retained_closing_portal() {
    let mut state = state();
    let portal = portal(261, 271);
    let opened = state.prepare(open_request(portal, 281)).unwrap();
    state.commit_published(opened).unwrap();

    let UiPortalDismissalPreparation::Prepared(first) = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::Escape {
                semantic_surface: state.semantic_surface_for_test(portal).unwrap(),
            },
            None,
            idempotency(282),
        )
        .unwrap()
    else {
        panic!("the first dismissal must prepare");
    };
    state
        .commit_published_with_exit_retention(first.into_transition(), true)
        .unwrap();
    let placement = state
        .placement(portal)
        .expect("a retained closing portal keeps its placement");

    let UiPortalDismissalPreparation::Prepared(second) = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::AcceptedSelection {
                semantic_surface: state.semantic_surface_for_test(portal).unwrap(),
            },
            None,
            idempotency(283),
        )
        .unwrap()
    else {
        panic!("a second non-anchor dismissal must republish a retained close");
    };
    let receipt = state
        .commit_published_with_exit_retention(second.into_transition(), true)
        .unwrap()
        .0;

    assert_eq!(receipt.posture(), UiPortalLifecyclePosture::Closing);
    assert_eq!(state.posture(portal), UiPortalLifecyclePosture::Closing);
    assert_eq!(state.placement(portal), Some(placement));
    assert_eq!(state.active_count(), 1);
    assert_eq!(state.exit_retention_count(), 1);
    assert_eq!(state.admitted_requests(), 3);
}

#[test]
fn modal_policy_shields_input_and_disables_outside_press_dismissal() {
    let mut state = super::UiPortalRuntimeState::new_with_policy(
        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
        crate::declaration::UiPortalPolicy::modal_dialog(),
    );
    let portal = portal(281, 291);
    let opened = state.prepare(open_request(portal, 301)).unwrap();
    assert_eq!(
        opened.placement().unwrap().shielding(),
        UiPortalInputShielding::ModalSurface
    );
    state.commit_published(opened).unwrap();

    assert!(matches!(
        state
            .prepare_dismissal(
                UiPortalDismissalTrigger::OutsidePress {
                    semantic_surface: state.semantic_surface_for_test(portal).unwrap(),
                    viewport_point_bits: [0.0_f32.to_bits(), 0.0_f32.to_bits()],
                },
                None,
                idempotency(302),
            )
            .unwrap(),
        UiPortalDismissalPreparation::Ignored(UiPortalDismissalIgnoreReason::NoMatchingPortal)
    ));
    assert_eq!(state.active_count(), 1);
}

#[test]
fn accepted_selection_and_anchor_loss_respect_the_declared_policy() {
    let policy = crate::declaration::UiPortalPolicy::dropdown()
        .with_accepted_selection_dismissal(false)
        .with_anchor_loss_dismissal(false);
    let mut state = super::UiPortalRuntimeState::new_with_policy(
        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
        policy,
    );
    let portal = portal(311, 321);
    let opened = state.prepare(open_request(portal, 331)).unwrap();
    state.commit_published(opened).unwrap();
    let surface = state.semantic_surface_for_test(portal).unwrap();

    for trigger in [
        UiPortalDismissalTrigger::AcceptedSelection {
            semantic_surface: surface,
        },
        UiPortalDismissalTrigger::AnchorLoss(portal),
    ] {
        assert!(matches!(
            state
                .prepare_dismissal(trigger, None, idempotency(332))
                .unwrap(),
            UiPortalDismissalPreparation::Ignored(UiPortalDismissalIgnoreReason::NoMatchingPortal)
        ));
    }
    assert_eq!(state.active_count(), 1);
}

#[test]
fn accepted_selection_closes_the_topmost_portal_with_its_typed_cause() {
    let mut state = state();
    let identity = portal(312, 322);
    let opened = state.prepare(open_request(identity, 333)).unwrap();
    state.commit_published(opened).unwrap();

    let UiPortalDismissalPreparation::Prepared(dismissal) = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::AcceptedSelection {
                semantic_surface: state.semantic_surface_for_test(identity).unwrap(),
            },
            None,
            idempotency(334),
        )
        .unwrap()
    else {
        panic!("the dropdown policy admits accepted-selection dismissal")
    };
    assert_eq!(dismissal.portal(), identity);
    state.commit_published(dismissal.into_transition()).unwrap();

    let closed = state
        .last_closed()
        .expect("accepted selection publishes inspectable close truth");
    assert_eq!(closed.portal(), identity);
    assert_eq!(closed.cause(), UiPortalDismissalCause::AcceptedSelection);
}
