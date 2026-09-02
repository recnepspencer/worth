use super::super::*;
use super::support::{declaration, input, portal_snapshot, presentation, surface_extent};

#[test]
fn reserves_capacity_before_publishing_a_snapshot() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let extent = surface_extent(surface, runtime_surface, 2);
    let portals = portal_snapshot(runtime_surface, []);
    let mut capacity = UiOverlayCapacityProfile::qualified();
    capacity.max_backdrop_rows = 0;
    let state = UiOverlayCompositionState::admit([backdrop], 3, capacity).unwrap();

    assert_eq!(
        state.prepare_initial(input(&extent, &portals, &[], None, presentation())),
        Err(UiOverlayCompositionDenial::BackdropRowCapacityExceeded {
            observed: 1,
            maximum: 0,
        })
    );
    assert!(state.current().is_none());
}

#[test]
fn missing_bindings_deny_the_candidate_without_fabricating_portal_rows() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let portal = super::support::portal(8, 9);
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let extent = surface_extent(surface, runtime_surface, 2);
    let snapshot = portal_snapshot(runtime_surface, [(portal, 1)]);
    let state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();

    assert_eq!(
        state.prepare_initial(input(&extent, &snapshot, &[], None, presentation())),
        Err(UiOverlayCompositionDenial::MissingPortalDeclarationBinding(
            portal
        ))
    );
}
