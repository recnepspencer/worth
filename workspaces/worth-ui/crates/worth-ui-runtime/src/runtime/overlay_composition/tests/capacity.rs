use super::super::*;
use super::support::{
    declaration, input, portal, portal_snapshot, portal_snapshot_with_rows, presentation,
    surface_extent,
};

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
    let portal = super::support::portal(8);
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

#[test]
fn indexed_portal_pass_reports_each_source_row_and_binding_once() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = portal(1);
    let second = portal(2);
    let third = portal(3);
    let snapshot = portal_snapshot(runtime_surface, [(first, 1), (second, 2), (third, 3)]);
    let declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let bindings = [
        UiOverlayPortalBinding::new(declaration, first),
        UiOverlayPortalBinding::new(declaration, second),
        UiOverlayPortalBinding::new(declaration, third),
    ];
    let extent = surface_extent(surface, runtime_surface, 2);
    let state = UiOverlayCompositionState::admit([], 3, Default::default()).unwrap();
    let prepared = state
        .prepare_initial(input(&extent, &snapshot, &bindings, None, presentation()))
        .unwrap();

    assert_eq!(prepared.counters().portal_stack_rows_read(), 3);
    assert_eq!(prepared.counters().portal_binding_entries_read(), 3);
    assert_eq!(prepared.reservation().portal_rows, 3);
}

#[test]
fn duplicate_source_rows_and_bindings_are_denied_before_publication() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let runtime_portal = portal(4);
    let declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let state = UiOverlayCompositionState::admit([], 3, Default::default()).unwrap();
    let duplicate_binding = [
        UiOverlayPortalBinding::new(declaration, runtime_portal),
        UiOverlayPortalBinding::new(declaration, runtime_portal),
    ];
    let snapshot = portal_snapshot(runtime_surface, [(runtime_portal, 1)]);
    assert_eq!(
        state.prepare_initial(input(
            &extent,
            &snapshot,
            &duplicate_binding,
            None,
            presentation(),
        )),
        Err(UiOverlayCompositionDenial::DuplicatePortalBinding)
    );

    let binding = [UiOverlayPortalBinding::new(declaration, runtime_portal)];
    let duplicate_snapshot =
        portal_snapshot(runtime_surface, [(runtime_portal, 1), (runtime_portal, 2)]);
    assert_eq!(
        state.prepare_initial(input(
            &extent,
            &duplicate_snapshot,
            &binding,
            None,
            presentation(),
        )),
        Err(UiOverlayCompositionDenial::DuplicatePortalSnapshotRow(
            runtime_portal
        ))
    );
}

#[test]
fn foreign_bound_portal_is_not_silently_skipped() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let target_surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let foreign_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let runtime_portal = portal(5);
    let declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let snapshot = portal_snapshot_with_rows(
        7,
        [(
            runtime_portal,
            None,
            foreign_surface,
            1,
            crate::runtime::portal::UiPortalLifecyclePosture::Visible,
        )],
    );
    let bindings = [UiOverlayPortalBinding::new(declaration, runtime_portal)];
    let extent = surface_extent(surface, target_surface, 2);
    let state = UiOverlayCompositionState::admit([], 3, Default::default()).unwrap();

    assert_eq!(
        state.prepare_initial(input(&extent, &snapshot, &bindings, None, presentation(),)),
        Err(UiOverlayCompositionDenial::ForeignPortalBinding {
            portal: runtime_portal,
            surface: foreign_surface,
        })
    );
}

#[test]
fn source_snapshot_capacity_is_denied_before_the_indexed_pass() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = portal(6);
    let second = portal(7);
    let third = portal(8);
    let snapshot = portal_snapshot(runtime_surface, [(first, 1), (second, 2), (third, 3)]);
    let declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let bindings = [
        UiOverlayPortalBinding::new(declaration, first),
        UiOverlayPortalBinding::new(declaration, second),
    ];
    let extent = surface_extent(surface, runtime_surface, 2);
    let mut capacity = UiOverlayCapacityProfile::qualified();
    capacity.max_portal_rows = 2;
    let state = UiOverlayCompositionState::admit([], 3, capacity).unwrap();

    assert_eq!(
        state.prepare_initial(input(&extent, &snapshot, &bindings, None, presentation())),
        Err(UiOverlayCompositionDenial::PortalRowCapacityExceeded {
            observed: 3,
            maximum: 2,
        })
    );
}
