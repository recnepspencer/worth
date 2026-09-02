use super::super::binding_export::UiOverlayBindingExportDenial;
use super::super::{UiOverlayPortalBinding, UiOverlayPortalBindingExport};
use super::owner_support::prepared_generation;
use super::support::portal;

#[test]
fn binding_export_accepts_repeated_declarations_and_preserves_portal_order() {
    let generation = prepared_generation();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = portal(403);
    let second = portal(401);
    let third = portal(402);
    let repeated_declaration = worth_ui_dsl::UiPortalDeclarationId::new(40).unwrap();
    let bindings = [
        UiOverlayPortalBinding::new(repeated_declaration, first),
        UiOverlayPortalBinding::new(repeated_declaration, second),
        UiOverlayPortalBinding::new(worth_ui_dsl::UiPortalDeclarationId::new(41).unwrap(), third),
    ];

    let export =
        UiOverlayPortalBindingExport::from_prepared(generation, runtime_surface, 7, bindings)
            .unwrap();

    assert_eq!(export.rows(), bindings.as_slice());
}

#[test]
fn binding_export_rejects_a_nonadjacent_duplicate_runtime_portal() {
    let generation = prepared_generation();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let duplicate = portal(404);
    let bindings = [
        UiOverlayPortalBinding::new(
            worth_ui_dsl::UiPortalDeclarationId::new(42).unwrap(),
            duplicate,
        ),
        UiOverlayPortalBinding::new(
            worth_ui_dsl::UiPortalDeclarationId::new(43).unwrap(),
            portal(405),
        ),
        UiOverlayPortalBinding::new(
            worth_ui_dsl::UiPortalDeclarationId::new(44).unwrap(),
            duplicate,
        ),
    ];

    assert_eq!(
        UiOverlayPortalBindingExport::from_prepared(generation, runtime_surface, 7, bindings),
        Err(UiOverlayBindingExportDenial::DuplicatePortal)
    );
}
