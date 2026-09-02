use super::super::*;
use super::support::{declaration, input, portal_snapshot, presentation, surface_extent};

#[test]
fn affected_scope_selects_only_direct_dependents_and_reports_reasons() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let first_portal = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let second_portal = worth_ui_dsl::UiPortalDeclarationId::new(11).unwrap();
    let first = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(first_portal),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(first_portal),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(first_portal),
    );
    let second = declaration(
        2,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(second_portal),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(second_portal),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(second_portal),
    );
    let first_identity = first.identity();
    let second_identity = second.identity();
    let index = UiOverlayDependencyIndex::rebuild(&[first, second]).unwrap();
    let affected = index.affected_scope(&UiOverlayChangeSet::from_changes([
        UiOverlayChangedBasis::PortalPresence(first_portal),
    ]));

    assert_eq!(affected.portals(), &[first_portal]);
    assert_eq!(affected.backdrops().len(), 1);
    assert_eq!(affected.backdrops()[0].identity(), first_identity);
    assert_eq!(
        affected.backdrops()[0].reasons(),
        &[UiOverlayDependencyKind::Presence]
    );
    assert!(affected
        .backdrops()
        .iter()
        .all(|row| row.identity() != second_identity));
}

#[test]
fn indexed_scope_presence_placement_and_motion_dependents_are_exact() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let portal = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let declarations = [
        declaration(
            1,
            surface,
            worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal),
            worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal),
            worth_ui_dsl::UiBackdropMotionBasis::None,
            worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
        ),
        declaration(
            2,
            surface,
            worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
            worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal),
            worth_ui_dsl::UiBackdropMotionBasis::None,
            worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
        ),
        declaration(
            3,
            surface,
            worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
            worth_ui_dsl::UiBackdropPresenceBasis::Always,
            worth_ui_dsl::UiBackdropMotionBasis::None,
            worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal),
        ),
        declaration(
            4,
            surface,
            worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
            worth_ui_dsl::UiBackdropPresenceBasis::Always,
            worth_ui_dsl::UiBackdropMotionBasis::PortalPresentation(portal),
            worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
        ),
    ];
    let index = UiOverlayDependencyIndex::rebuild(&declarations).unwrap();
    let cases = [
        (
            UiOverlayChangedBasis::PortalScope(portal),
            worth_ui_dsl::UiBackdropIdentity::new(1).unwrap(),
            UiOverlayDependencyKind::Scope,
        ),
        (
            UiOverlayChangedBasis::PortalPresence(portal),
            worth_ui_dsl::UiBackdropIdentity::new(2).unwrap(),
            UiOverlayDependencyKind::Presence,
        ),
        (
            UiOverlayChangedBasis::PortalPlacement(portal),
            worth_ui_dsl::UiBackdropIdentity::new(3).unwrap(),
            UiOverlayDependencyKind::Placement,
        ),
        (
            UiOverlayChangedBasis::PortalMotion(portal),
            worth_ui_dsl::UiBackdropIdentity::new(4).unwrap(),
            UiOverlayDependencyKind::Motion,
        ),
    ];
    for (basis, identity, reason) in cases {
        let affected = index.affected_scope(&UiOverlayChangeSet::from_changes([basis]));
        assert_eq!(affected.backdrops().len(), 1);
        assert_eq!(affected.backdrops()[0].identity(), identity);
        assert_eq!(affected.backdrops()[0].reasons(), &[reason]);
    }
}

#[test]
fn reconstruction_rebuilds_the_index_and_preserves_the_published_snapshot() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let portals = portal_snapshot(runtime_surface, []);
    let presentation = presentation();
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let mut state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();
    let initial = state
        .prepare_initial(input(&extent, &portals, &[], None, presentation))
        .unwrap();
    state.publish(initial).unwrap();
    let published = state.current().unwrap().clone();
    state.discard_index_for_test();
    state.discard_relation_cache_for_test();

    let reconstructed = state
        .reconstruct(input(&extent, &portals, &[], None, presentation))
        .unwrap();
    assert_eq!(reconstructed.snapshot(), &published);
    assert_eq!(reconstructed.counters().backdrop_declarations_selected(), 1);
    state.publish(reconstructed).unwrap();
    assert!(state.dependency_index().is_some());
}

#[test]
fn stale_successor_is_denied_and_the_current_snapshot_is_retained() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let next_extent = super::support::surface_extent_with_viewport(
        surface,
        runtime_surface,
        3,
        super::support::box_at(0.0, 0.0, 640.0, 480.0),
        [],
    );
    let final_extent = super::support::surface_extent_with_viewport(
        surface,
        runtime_surface,
        4,
        super::support::box_at(0.0, 0.0, 320.0, 240.0),
        [],
    );
    let portals = portal_snapshot(runtime_surface, []);
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let mut state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();
    let initial = state
        .prepare_initial(input(&extent, &portals, &[], None, presentation()))
        .unwrap();
    state.publish(initial).unwrap();
    let changes = UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::SurfaceExtent(surface)]);
    let next = state
        .prepare_successor(
            input(&next_extent, &portals, &[], None, presentation()),
            &changes,
        )
        .unwrap();
    let stale = state
        .prepare_successor(
            input(&final_extent, &portals, &[], None, presentation()),
            &changes,
        )
        .unwrap();
    state.publish(next).unwrap();
    assert_eq!(
        state.publish(stale),
        Err(UiOverlayCommitDenial::StalePredecessor)
    );
    assert_eq!(state.current().unwrap().extent_revision(), 3);
}

#[test]
fn structural_successor_recomputes_the_indexed_scope_only() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let runtime_portal = super::support::portal(20);
    let extent = surface_extent(surface, runtime_surface, 2);
    let portals = portal_snapshot(runtime_surface, [(runtime_portal, 1)]);
    let selected = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    );
    let unrelated = declaration(
        2,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterPortal(portal_declaration),
    );
    let bindings = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    let mut state =
        UiOverlayCompositionState::admit([selected, unrelated], 3, Default::default()).unwrap();
    let initial = state
        .prepare_initial(input(&extent, &portals, &bindings, None, presentation()))
        .unwrap();
    state.publish(initial).unwrap();
    let changes = UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::PortalPresence(
        portal_declaration,
    )]);

    let successor = state
        .prepare_successor(
            input(&extent, &portals, &bindings, None, presentation()),
            &changes,
        )
        .unwrap();
    assert_eq!(successor.counters().backdrop_declarations_selected(), 1);
    assert_eq!(successor.counters().unrelated_neighborhoods_touched(), 0);
    assert_eq!(successor.snapshot().participants().len(), 3);
}
