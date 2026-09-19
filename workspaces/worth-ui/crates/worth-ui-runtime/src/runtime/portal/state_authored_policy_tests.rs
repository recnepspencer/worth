use super::{
    test_support::{
        idempotency, portal, presented_geometry, semantic_surface, state, viewport_bounds,
    },
    UiPortalDismissalIgnoreReason, UiPortalDismissalPreparation, UiPortalDismissalTrigger,
    UiPortalInputShielding, UiPortalServiceRequest,
};

#[test]
fn each_authored_portal_row_owns_its_shielding_and_dismissal_policy() {
    let mut state = state();
    let surface = semantic_surface();
    let lower = portal(401, 411);
    let lower_geometry = presented_geometry(1);
    let lower_request = UiPortalServiceRequest::open(
        lower,
        idempotency(421),
        lower_geometry,
        Some(viewport_bounds(lower_geometry)),
        surface,
    )
    .with_declared_portal(Some(worth_ui_dsl::UiPortalDeclarationId::new(1).unwrap()));
    let lower_open = state
        .prepare_authored(
            lower_request,
            crate::declaration::UiPortalPolicy::dropdown(),
        )
        .unwrap();
    assert_eq!(
        lower_open.placement().unwrap().shielding(),
        UiPortalInputShielding::ContentBounds
    );
    state.commit_published(lower_open).unwrap();

    let upper = portal(402, 412);
    let upper_geometry = presented_geometry(2);
    let upper_request = UiPortalServiceRequest::open_nested(
        upper,
        idempotency(422),
        upper_geometry,
        viewport_bounds(upper_geometry),
        surface,
        lower,
        UiPortalInputShielding::ContentBounds,
    )
    .with_declared_portal(Some(worth_ui_dsl::UiPortalDeclarationId::new(2).unwrap()));
    let modal = crate::declaration::UiPortalPolicy::modal_dialog()
        .with_escape_dismissal(false)
        .with_outside_press_dismissal(false);
    let upper_open = state.prepare_authored(upper_request, modal).unwrap();
    assert_eq!(
        upper_open.placement().unwrap().shielding(),
        UiPortalInputShielding::ModalSurface
    );
    state.commit_published(upper_open).unwrap();

    assert!(matches!(
        state
            .prepare_dismissal(
                UiPortalDismissalTrigger::Escape {
                    semantic_surface: surface
                },
                None,
                idempotency(423),
            )
            .unwrap(),
        UiPortalDismissalPreparation::Ignored(UiPortalDismissalIgnoreReason::NoMatchingPortal)
    ));
    assert_eq!(state.active_count(), 2);

    let upper_close = state
        .prepare(UiPortalServiceRequest::close(
            upper,
            idempotency(424),
            super::UiPortalDismissalCause::ExplicitOwnerRequest,
            surface,
        ))
        .unwrap();
    state.commit_published(upper_close).unwrap();
    let UiPortalDismissalPreparation::Prepared(lower_escape) = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::Escape {
                semantic_surface: surface,
            },
            None,
            idempotency(425),
        )
        .unwrap()
    else {
        panic!("the lower authored dropdown regains topmost dismissal authority");
    };
    assert_eq!(lower_escape.portal(), lower);
    assert!(!lower_escape.input_shielded());
}
