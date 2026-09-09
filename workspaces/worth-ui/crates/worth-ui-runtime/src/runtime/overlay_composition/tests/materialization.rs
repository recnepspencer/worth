use super::super::*;
use super::support::{
    box_at, declaration, declaration_with_extent, input, portal, portal_snapshot, presentation,
    surface_extent, surface_extent_with_viewport,
};

#[test]
fn materializes_a_per_portal_backdrop_with_an_incarnation_identity() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let runtime_portal = portal(20);
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    );
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let portals = portal_snapshot(runtime_surface, [(runtime_portal, 1)]);
    let bindings = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    let state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();

    let prepared = state
        .prepare_initial(input(&extent, &portals, &bindings, None, presentation()))
        .unwrap();
    let participants = prepared.snapshot().participants();
    assert_eq!(participants.len(), 2);
    assert!(matches!(
        &participants[0],
        UiOverlayStackParticipant::Backdrop(row)
            if row.identity().scope() == UiOverlayBackdropInstanceScope::Portal(runtime_portal)
    ));
    assert!(matches!(
        &participants[1],
        UiOverlayStackParticipant::Portal(row) if row.portal() == runtime_portal
    ));
    assert_eq!(prepared.reservation().backdrop_rows, 1);
    assert_eq!(prepared.counters().backdrop_mechanics_changed(), 1);
}

#[test]
fn repeats_per_portal_rows_for_distinct_current_incarnations() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let first_portal = portal(20);
    let second_portal = portal(20);
    assert_ne!(first_portal, second_portal);
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    );
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let portals = portal_snapshot(runtime_surface, [(first_portal, 1), (second_portal, 2)]);
    let bindings = [
        UiOverlayPortalBinding::new(portal_declaration, first_portal),
        UiOverlayPortalBinding::new(portal_declaration, second_portal),
    ];
    let state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();
    let prepared = state
        .prepare_initial(input(&extent, &portals, &bindings, None, presentation()))
        .unwrap();

    let identities = prepared
        .snapshot()
        .participants()
        .iter()
        .map(UiOverlayStackParticipant::identity)
        .collect::<Vec<_>>();
    assert_eq!(
        identities,
        vec![
            UiOverlayParticipantIdentity::Backdrop(UiBackdropInstanceIdentity::new(
                worth_ui_dsl::UiBackdropIdentity::new(1).unwrap(),
                UiOverlayBackdropInstanceScope::Portal(first_portal),
            )),
            UiOverlayParticipantIdentity::Portal(first_portal),
            UiOverlayParticipantIdentity::Backdrop(UiBackdropInstanceIdentity::new(
                worth_ui_dsl::UiBackdropIdentity::new(1).unwrap(),
                UiOverlayBackdropInstanceScope::Portal(second_portal),
            )),
            UiOverlayParticipantIdentity::Portal(second_portal),
        ]
    );
}

#[test]
fn resolves_region_extent_and_optional_motion_without_host_work() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let region = worth_ui_dsl::UiMosaicRegionDeclarationIdentity::new(5).unwrap();
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let runtime_portal = portal(20);
    let backdrop = declaration_with_extent(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropExtentBasis::PresentedMosaicRegion { surface, region },
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::PortalPresentation(portal_declaration),
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    );
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent_with_viewport(
        surface,
        runtime_surface,
        2,
        box_at(0.0, 0.0, 800.0, 600.0),
        [UiOverlayRegionExtent::new(
            region,
            runtime_portal.owner().mounted_instance_identity(),
            box_at(10.0, 20.0, 300.0, 200.0),
        )],
    );
    let portals = portal_snapshot(runtime_surface, [(runtime_portal, 1)]);
    let bindings = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    let motion = UiOverlayMotionSnapshot::seal(
        9,
        [UiOverlayMotionBinding::new(
            portal_declaration,
            runtime_portal,
            4,
        )],
    )
    .unwrap();
    let state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();
    let prepared = state
        .prepare_initial(input(
            &extent,
            &portals,
            &bindings,
            Some(&motion),
            presentation(),
        ))
        .unwrap();
    let UiOverlayStackParticipant::Backdrop(row) = &prepared.snapshot().participants()[0] else {
        panic!("expected backdrop before Portal")
    };
    assert_eq!(row.extent().bounds(), box_at(10.0, 20.0, 300.0, 200.0));
    assert_eq!(row.motion().unwrap().revision(), 4);
    assert_eq!(prepared.snapshot().motion_revision(), Some(9));
}
