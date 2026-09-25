use super::*;

#[test]
fn pressing_the_portal_anchor_preserves_open_activation_and_its_half_open_boundary() {
    let mut state = state();
    let portal = portal(751, 761);
    let geometry = presented_geometry(1);
    let surface = semantic_surface();
    let request = |lineage| {
        UiPortalServiceRequest::open(
            portal,
            idempotency(lineage),
            geometry,
            Some(viewport_bounds(geometry)),
            surface,
        )
    };
    let open = state.prepare(request(771)).unwrap();
    let anchor = open.placement().unwrap().anchor().canonical_box();
    state.commit_published(open).unwrap();
    let press = |x: f32, y: f32| UiPortalDismissalTrigger::OutsidePress {
        semantic_surface: surface,
        point: crate::mounting::presentation::platform_point_for_test(x, y),
    };
    assert!(matches!(
        state
            .prepare_dismissal(press(anchor.x(), anchor.y()), None, idempotency(772),)
            .unwrap(),
        UiPortalDismissalPreparation::Ignored(UiPortalDismissalIgnoreReason::InsideTopmostPortal)
    ));
    assert_eq!(state.posture(portal), UiPortalLifecyclePosture::Visible);
    let reopen = state.prepare(request(773)).unwrap();
    state.commit_published(reopen).unwrap();
    assert_eq!(state.posture(portal), UiPortalLifecyclePosture::Visible);
    assert!(
        matches!(
            state
                .prepare_dismissal(
                    press(anchor.x() + anchor.width(), anchor.y()),
                    None,
                    idempotency(774),
                )
                .unwrap(),
            UiPortalDismissalPreparation::Prepared(_)
        ),
        "the anchor's excluded right edge remains outside; unrelated outside dismissal still works"
    );
}
