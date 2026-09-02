use super::super::{
    UiOverlayChangeSet, UiOverlayChangedBasis, UiOverlayCompositionOwnerLifecycle,
    UiOverlayOwnerBridgeDenial, UiOverlayOwnerSources,
};
use super::owner_support::{commit_open, open_request, owner_portal, owner_state};
use super::support::declaration;
use crate::runtime::allocation_receipt::{
    UiCommittedOverlayExtentBounds, UiMountedOverlayExtentOwner,
};
use crate::runtime::motion::UiMotionRuntimeState;
use crate::runtime::portal::UiPortalOverlayBindingOwner;
use crate::runtime::presentation_state::UiApplicationPresentationState;

fn sources<'a>(
    generation: &'a crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationGenerationIdentity,
    portal: &'a crate::runtime::portal::UiPortalRuntimeState,
    extent: &'a UiMountedOverlayExtentOwner,
    presentation: &'a crate::runtime::presentation_state::UiApplicationPresentationOwnerExport,
    bindings: &'a UiPortalOverlayBindingOwner,
    motion: &'a UiMotionRuntimeState,
) -> UiOverlayOwnerSources<'a> {
    UiOverlayOwnerSources {
        generation,
        portal,
        extent,
        presentation,
        bindings,
        motion,
    }
}

fn prepared_owner() -> crate::facade::entry::WorthUiHostNeutralApp {
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .expect("production owner fixture freezes without a host")
}

fn owner_bounds(x: f32, y: f32, width: f32, height: f32) -> UiCommittedOverlayExtentBounds {
    UiCommittedOverlayExtentBounds::new(x, y, width, height).unwrap()
}

fn backdrop(
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    portal: worth_ui_dsl::UiPortalDeclarationId,
) -> worth_ui_dsl::UiBackdropDeclaration {
    declaration(
        901,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal),
    )
}

#[test]
fn normal_library_lifecycle_reaches_actual_owner_exports() {
    let app = prepared_owner();
    let generation = app.generation_identity().clone();
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(90).unwrap();
    let declaration = worth_ui_dsl::UiPortalDeclarationId::new(900).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let portal = owner_portal(91);
    let mut portal_owner = owner_state();
    commit_open(&mut portal_owner, open_request(portal, runtime_surface, 90));
    let extent = UiMountedOverlayExtentOwner::new(
        generation.clone(),
        surface,
        runtime_surface,
        1,
        owner_bounds(0.0, 0.0, 800.0, 600.0),
        [],
    )
    .unwrap();
    let presentation_state = UiApplicationPresentationState::activate(app.capabilities());
    let presentation = presentation_state.overlay_owner_export(
        generation.clone(),
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
    );
    let mut bindings = UiPortalOverlayBindingOwner::new(generation.clone(), runtime_surface);
    bindings.bind(declaration, portal).unwrap();
    let motion =
        UiMotionRuntimeState::new(crate::runtime::UiServiceStatePersistencePosture::Ephemeral);
    let lifecycle = UiOverlayCompositionOwnerLifecycle::admit_from_owners(
        [backdrop(surface, declaration)],
        1,
        sources(
            &generation,
            &portal_owner,
            &extent,
            &presentation,
            &bindings,
            &motion,
        ),
    )
    .expect("production owner lifecycle retains the first derived snapshot");
    assert_eq!(
        lifecycle.current().unwrap().portal_revision(),
        portal_owner.revision()
    );
    let reconstructed = lifecycle
        .reconstruct_from_owners(sources(
            &generation,
            &portal_owner,
            &extent,
            &presentation,
            &bindings,
            &motion,
        ))
        .expect("production bridge reconstructs from owner exports");
    assert_eq!(
        reconstructed.snapshot(),
        lifecycle.current().expect("retained derived snapshot")
    );
}

#[test]
fn normal_library_lifecycle_rejects_incoherent_owner_generation_before_planning() {
    let app = prepared_owner();
    let generation = app.generation_identity().clone();
    let foreign_generation = super::owner_support::prepared_generation_variant();
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(92).unwrap();
    let declaration = worth_ui_dsl::UiPortalDeclarationId::new(920).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let portal = owner_portal(93);
    let mut portal_owner = owner_state();
    commit_open(&mut portal_owner, open_request(portal, runtime_surface, 92));
    let foreign_extent = UiMountedOverlayExtentOwner::new(
        foreign_generation,
        surface,
        runtime_surface,
        1,
        owner_bounds(0.0, 0.0, 800.0, 600.0),
        [],
    )
    .unwrap();
    let extent = UiMountedOverlayExtentOwner::new(
        generation.clone(),
        surface,
        runtime_surface,
        1,
        owner_bounds(0.0, 0.0, 800.0, 600.0),
        [],
    )
    .unwrap();
    let presentation_state = UiApplicationPresentationState::activate(app.capabilities());
    let presentation = presentation_state.overlay_owner_export(
        generation.clone(),
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
    );
    let mut bindings = UiPortalOverlayBindingOwner::new(generation.clone(), runtime_surface);
    bindings.bind(declaration, portal).unwrap();
    let motion =
        UiMotionRuntimeState::new(crate::runtime::UiServiceStatePersistencePosture::Ephemeral);
    let lifecycle = UiOverlayCompositionOwnerLifecycle::admit_from_owners(
        [backdrop(surface, declaration)],
        1,
        sources(
            &generation,
            &portal_owner,
            &extent,
            &presentation,
            &bindings,
            &motion,
        ),
    )
    .unwrap();
    let before = lifecycle.current().cloned().unwrap();
    let denied = lifecycle.prepare_successor_from_owners(
        sources(
            &generation,
            &portal_owner,
            &foreign_extent,
            &presentation,
            &bindings,
            &motion,
        ),
        &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::SurfaceExtent(surface)]),
    );
    assert!(matches!(
        denied,
        Err(UiOverlayOwnerBridgeDenial::OwnerExports(
            super::super::UiOverlayOwnerExportDenial::ExtentGenerationMismatch
        ))
    ));
    assert_eq!(lifecycle.current(), Some(&before));
}

#[test]
fn repeated_portal_instances_share_one_declaration_in_owner_export() {
    let generation = prepared_owner().generation_identity().clone();
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let declaration = worth_ui_dsl::UiPortalDeclarationId::new(940).unwrap();
    let original = owner_portal(94);
    let replacement = owner_portal(95);
    let mut portal_owner = owner_state();
    commit_open(&mut portal_owner, open_request(original, surface, 94));
    commit_open(&mut portal_owner, open_request(replacement, surface, 95));

    let mut bindings = UiPortalOverlayBindingOwner::new(generation, surface);
    bindings.bind(declaration, original).unwrap();
    bindings
        .bind(declaration, replacement)
        .expect("distinct Portal identities may share one declaration");

    let export = bindings
        .export(&portal_owner.stack_snapshot())
        .expect("shared declaration bindings remain exportable");
    assert_eq!(export.rows().len(), 2);
    assert_eq!(export.rows()[0].declaration(), declaration);
    assert_eq!(export.rows()[0].portal(), original);
    assert_eq!(export.rows()[1].declaration(), declaration);
    assert_eq!(export.rows()[1].portal(), replacement);
}
