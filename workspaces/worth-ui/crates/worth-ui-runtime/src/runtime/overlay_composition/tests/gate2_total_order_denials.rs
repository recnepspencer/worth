use super::super::*;
use super::support::{declaration, input, portal, portal_snapshot, presentation, surface_extent};

#[test]
fn gate2_total_order_denials_cover_graph_and_capacity_edges() {
    assert_ambiguous_tie_is_denied();
    assert_cycle_is_denied();
    assert_cross_scope_anchor_is_denied();
    assert_missing_backdrop_anchor_is_denied();
    assert_missing_portal_anchor_is_denied();
    assert_order_capacity_is_denied();
    assert_relation_edge_capacity_is_denied();
}

fn assert_ambiguous_tie_is_denied() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(91).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let empty_portals = portal_snapshot(runtime_surface, []);
    let tie_first = declaration(
        911,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let tie_second = declaration(
        912,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let state =
        UiOverlayCompositionState::admit([tie_first, tie_second], 3, Default::default()).unwrap();
    assert_eq!(
        state.prepare_initial(input(&extent, &empty_portals, &[], None, presentation(),)),
        Err(UiOverlayCompositionDenial::AmbiguousOrder)
    );
}

fn assert_cycle_is_denied() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(91).unwrap();
    let cycle_first = declaration(
        913,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(
            worth_ui_dsl::UiBackdropIdentity::new(914).unwrap(),
        ),
    );
    let cycle_second = declaration(
        914,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(
            worth_ui_dsl::UiBackdropIdentity::new(913).unwrap(),
        ),
    );
    let denial = match UiOverlayCompositionState::admit(
        [cycle_first, cycle_second],
        3,
        Default::default(),
    ) {
        Ok(_) => panic!("cycle must be denied during relation admission"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial,
        UiOverlayCompositionDenial::Relation(UiOverlayRelationCompilationDenial::Admission(
            worth_ui_dsl::UiOverlayRelationAdmissionDenial::Cycle,
        ))
    );
}

fn assert_cross_scope_anchor_is_denied() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(91).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let target = declaration(
        915,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(915).unwrap();
    let cross_scope = declaration(
        916,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(target.identity()),
    );
    let state = UiOverlayCompositionState::admit(
        [target.clone(), cross_scope.clone()],
        3,
        Default::default(),
    )
    .unwrap();
    let runtime_portal = portal(917);
    let portals = portal_snapshot(runtime_surface, [(runtime_portal, 1)]);
    let bindings = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    assert_eq!(
        state.prepare_initial(input(&extent, &portals, &bindings, None, presentation(),)),
        Err(UiOverlayCompositionDenial::CrossScopeBackdropAnchor {
            backdrop: cross_scope.identity(),
            anchor: target.identity(),
        })
    );
}

fn assert_missing_backdrop_anchor_is_denied() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(91).unwrap();
    let missing_backdrop = declaration(
        918,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(
            worth_ui_dsl::UiBackdropIdentity::new(919).unwrap(),
        ),
    );
    let denial = match UiOverlayCompositionState::admit([missing_backdrop], 3, Default::default()) {
        Ok(_) => panic!("missing backdrop anchor must be denied during admission"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial,
        UiOverlayCompositionDenial::Relation(UiOverlayRelationCompilationDenial::Admission(
            worth_ui_dsl::UiOverlayRelationAdmissionDenial::MissingAnchor,
        ))
    );
}

fn assert_missing_portal_anchor_is_denied() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(91).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let empty_portals = portal_snapshot(runtime_surface, []);
    let missing_portal = worth_ui_dsl::UiPortalDeclarationId::new(920).unwrap();
    let backdrop = declaration(
        920,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(missing_portal),
    );
    let state =
        UiOverlayCompositionState::admit([backdrop.clone()], 3, Default::default()).unwrap();
    assert_eq!(
        state.prepare_initial(input(&extent, &empty_portals, &[], None, presentation(),)),
        Err(UiOverlayCompositionDenial::MissingPortalAnchor {
            backdrop: backdrop.identity(),
            portal: missing_portal,
        })
    );
}

fn assert_order_capacity_is_denied() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(91).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let runtime_portal = portal(921);
    let portals = portal_snapshot(runtime_surface, [(runtime_portal, 1)]);
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(921).unwrap();
    let bindings = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    let mut capacity = UiOverlayCapacityProfile::qualified();
    capacity.max_order_rows = 0;
    let state = UiOverlayCompositionState::admit([], 3, capacity).unwrap();
    assert_eq!(
        state.prepare_initial(input(&extent, &portals, &bindings, None, presentation(),)),
        Err(UiOverlayCompositionDenial::OverlayOrderCapacityExceeded {
            observed: 1,
            maximum: 0,
        })
    );
}

fn assert_relation_edge_capacity_is_denied() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(91).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let runtime_portal = portal(921);
    let portals = portal_snapshot(runtime_surface, [(runtime_portal, 1)]);
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(921).unwrap();
    let bindings = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    let mut capacity = UiOverlayCapacityProfile::qualified();
    capacity.max_relation_edges = 0;
    let state = UiOverlayCompositionState::admit([], 3, capacity).unwrap();
    assert_eq!(
        state.prepare_initial(input(&extent, &portals, &bindings, None, presentation(),)),
        Err(UiOverlayCompositionDenial::RelationEdgeCapacityExceeded {
            observed: 1,
            maximum: 0,
        })
    );
}
