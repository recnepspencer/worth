use super::super::*;
use super::support::{declaration, input, portal, portal_snapshot, presentation, surface_extent};

#[test]
fn surface_scrim_is_below_both_portals_only_with_explicit_adjacency() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(91).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let signals = portal(920);
    let review = portal(910);
    let signals_declaration = worth_ui_dsl::UiPortalDeclarationId::new(920).unwrap();
    let review_declaration = worth_ui_dsl::UiPortalDeclarationId::new(910).unwrap();
    let bindings = [
        UiOverlayPortalBinding::new(signals_declaration, signals),
        UiOverlayPortalBinding::new(review_declaration, review),
    ];
    // Deliberately reverse identity order: the Portal owner supplies stack order.
    let portals = portal_snapshot(runtime_surface, [(signals, 1), (review, 2)]);
    let scrim = |placement| {
        declaration(
            930,
            surface,
            worth_ui_dsl::UiBackdropScope::PerPortalInstance(review_declaration),
            worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(review_declaration),
            worth_ui_dsl::UiBackdropMotionBasis::None,
            placement,
        )
    };
    let prepare = |placement| {
        UiOverlayCompositionState::admit([scrim(placement)], 3, Default::default())
            .unwrap()
            .prepare_initial(input(&extent, &portals, &bindings, None, presentation()))
    };
    assert_eq!(
        prepare(worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent),
        Err(UiOverlayCompositionDenial::AmbiguousOrder),
    );
    let prepared = prepare(worth_ui_dsl::UiBackdropPlacement::ImmediatelyAboveSurfaceContent)
        .expect("surface scrim must sit below Signals and Review");
    let rows = prepared.snapshot().participants();
    assert_eq!(rows.len(), 3);
    assert!(matches!(&rows[0], UiOverlayStackParticipant::Backdrop(row)
        if row.declaration().value() == 930));
    assert!(matches!(&rows[1], UiOverlayStackParticipant::Portal(row)
        if row.portal() == signals));
    assert!(matches!(&rows[2], UiOverlayStackParticipant::Portal(row)
        if row.portal() == review));
}
