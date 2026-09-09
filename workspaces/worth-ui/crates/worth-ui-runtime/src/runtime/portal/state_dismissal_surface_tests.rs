use super::{
    test_support::{idempotency, portal, presented_geometry, state, viewport_bounds},
    UiPortalDismissalPreparation, UiPortalDismissalTrigger, UiPortalServiceRequest,
};

#[test]
fn interaction_dismissal_selects_the_topmost_portal_on_its_own_surface() {
    let mut state = state();
    let surface_a = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let surface_b = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let portal_a = portal(701, 711);
    let portal_b = portal(702, 712);
    open_on_surface(&mut state, portal_a, surface_a, 721, 1);
    open_on_surface(&mut state, portal_b, surface_b, 722, 2);

    let escape = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::Escape {
                semantic_surface: surface_a,
            },
            None,
            idempotency(723),
        )
        .unwrap();
    let UiPortalDismissalPreparation::Prepared(escape) = escape else {
        panic!("surface A Escape must find surface A's Portal");
    };
    assert_eq!(escape.portal(), portal_a);

    let outside = state
        .prepare_dismissal(
            UiPortalDismissalTrigger::OutsidePress {
                semantic_surface: surface_a,
                viewport_point_bits: [f32::MAX.to_bits(), f32::MAX.to_bits()],
            },
            None,
            idempotency(724),
        )
        .unwrap();
    let UiPortalDismissalPreparation::Prepared(outside) = outside else {
        panic!("surface A outside press must find surface A's Portal");
    };
    assert_eq!(outside.portal(), portal_a);

    assert_eq!(
        state.dismissal_target_identity(UiPortalDismissalTrigger::AcceptedSelection {
            semantic_surface: surface_a,
        }),
        Some(portal_a)
    );
    assert_eq!(
        state.dismissal_target_identity(UiPortalDismissalTrigger::Escape {
            semantic_surface: surface_b,
        }),
        Some(portal_b)
    );
}

fn open_on_surface(
    state: &mut super::UiPortalRuntimeState,
    portal: super::UiPortalIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    lineage: u64,
    epoch: u64,
) {
    let geometry = presented_geometry(epoch);
    let request = UiPortalServiceRequest::open(
        portal,
        idempotency(lineage),
        geometry,
        Some(viewport_bounds(geometry)),
        surface,
    );
    let opened = state.prepare(request).unwrap();
    state.commit_published(opened).unwrap();
}
