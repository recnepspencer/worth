use super::super::{
    UiOverlayChangeSet, UiOverlayChangedBasis, UiOverlayCompositionDenial,
    UiOverlayCompositionOwner, UiOverlayMotionBinding, UiOverlayOwnerExportDenial,
    UiOverlayOwnerExportVector, UiOverlayPortalBinding, UiOverlayPortalBindingExport,
    UiOverlayPortalOwnerExport, UiOverlayStackParticipant,
};
use super::owner_support::{
    close_request, commit_open, motion_export, nested_open_request, open_request, owner_exports,
    owner_portal, owner_state, prepared_generation, prepared_generation_variant,
};
use super::support::{declaration, presentation, surface_extent, surface_extent_with_viewport};
use crate::runtime::portal::UiPortalRuntimeState;

fn close(
    owner: &mut UiPortalRuntimeState,
    request: crate::runtime::portal::UiPortalServiceRequest,
) {
    let transition = owner.prepare(request).expect("owner close prepares");
    owner
        .commit_published(transition)
        .expect("owner close commits");
}

#[test]
fn owner_bridge_recomputes_exact_scope_after_parent_close() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(31).unwrap();
    let parent_declaration = worth_ui_dsl::UiPortalDeclarationId::new(310).unwrap();
    let child_declaration = worth_ui_dsl::UiPortalDeclarationId::new(311).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let parent = owner_portal(31);
    let child = owner_portal(33);
    let declarations = [
        declaration(
            31,
            surface,
            worth_ui_dsl::UiBackdropScope::PerPortalInstance(parent_declaration),
            worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(parent_declaration),
            worth_ui_dsl::UiBackdropMotionBasis::None,
            worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(parent_declaration),
        ),
        declaration(
            32,
            surface,
            worth_ui_dsl::UiBackdropScope::PerPortalInstance(child_declaration),
            worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(child_declaration),
            worth_ui_dsl::UiBackdropMotionBasis::None,
            worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(child_declaration),
        ),
    ];
    let generation = prepared_generation();
    let extent = surface_extent(surface, runtime_surface, 2);
    let presentation = presentation();
    let mut portal_owner = owner_state();
    commit_open(&mut portal_owner, open_request(parent, runtime_surface, 31));
    commit_open(
        &mut portal_owner,
        nested_open_request(child, parent, runtime_surface, 32),
    );
    let mut composition = UiOverlayCompositionOwner::admit(declarations, 3)
        .expect("production owner admits nested declarations");
    let initial = composition
        .prepare_initial(&owner_exports(
            &generation,
            &portal_owner,
            extent.clone(),
            presentation,
            [
                UiOverlayPortalBinding::new(parent_declaration, parent),
                UiOverlayPortalBinding::new(child_declaration, child),
            ],
            None,
        ))
        .unwrap();
    assert_eq!(
        initial
            .snapshot()
            .participants()
            .iter()
            .filter(|participant| matches!(participant, UiOverlayStackParticipant::Backdrop(_)))
            .count(),
        2
    );
    composition.retain_prepared(initial).unwrap();

    close(
        &mut portal_owner,
        close_request(parent, runtime_surface, 33),
    );
    assert_eq!(portal_owner.active_count(), 0);
    let successor = composition
        .prepare_successor(
            &owner_exports(&generation, &portal_owner, extent, presentation, [], None),
            &UiOverlayChangeSet::from_changes([
                UiOverlayChangedBasis::Portal(parent_declaration),
                UiOverlayChangedBasis::Portal(child_declaration),
            ]),
        )
        .expect("parent close successor uses the exact owner scope");
    assert!(successor.snapshot().participants().is_empty());
}

#[test]
fn owner_bridge_recomputes_extent_dependents_without_reconstructing_portals() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(32).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let generation = prepared_generation();
    let presentation = presentation();
    let old_extent = surface_extent(surface, runtime_surface, 2);
    let new_extent = surface_extent_with_viewport(
        surface,
        runtime_surface,
        3,
        super::support::box_at(0.0, 0.0, 1_024.0, 768.0),
        [],
    );
    let backdrop = declaration(
        33,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let portal_owner = owner_state();
    let mut composition = UiOverlayCompositionOwner::admit([backdrop], 3).unwrap();
    let initial = composition
        .prepare_initial(&owner_exports(
            &generation,
            &portal_owner,
            old_extent,
            presentation,
            [],
            None,
        ))
        .unwrap();
    composition.retain_prepared(initial).unwrap();
    let successor = composition
        .prepare_successor(
            &owner_exports(
                &generation,
                &portal_owner,
                new_extent,
                presentation,
                [],
                None,
            ),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::SurfaceExtent(surface)]),
        )
        .expect("resize successor uses the extent dependency");
    assert_eq!(successor.snapshot().extent_revision(), 3);
    assert_eq!(successor.snapshot().portal_revision(), 0);
    assert_eq!(successor.counters().portal_stack_rows_read(), 0);
}

#[test]
fn owner_bridge_reconstructs_a_rebound_generation_and_presentation() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(33).unwrap();
    let declaration_id = worth_ui_dsl::UiPortalDeclarationId::new(330).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let portal = owner_portal(35);
    let backdrop = declaration(
        34,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(declaration_id),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(declaration_id),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(declaration_id),
    );
    let mut portal_owner = owner_state();
    commit_open(&mut portal_owner, open_request(portal, runtime_surface, 34));
    let old_generation = prepared_generation();
    let new_generation = prepared_generation_variant();
    assert_ne!(old_generation, new_generation);
    let old_extent = surface_extent(surface, runtime_surface, 2);
    let new_extent = surface_extent(surface, runtime_surface, 3);
    let old_presentation = presentation();
    let new_presentation = presentation();
    let binding = [UiOverlayPortalBinding::new(declaration_id, portal)];
    let mut composition = UiOverlayCompositionOwner::admit([backdrop], 3).unwrap();
    let initial = composition
        .prepare_initial(&owner_exports(
            &old_generation,
            &portal_owner,
            old_extent,
            old_presentation,
            binding,
            None,
        ))
        .unwrap();
    composition.retain_prepared(initial).unwrap();
    let rebound_exports = owner_exports(
        &new_generation,
        &portal_owner,
        new_extent,
        new_presentation,
        binding,
        None,
    );
    assert!(matches!(
        composition.prepare_successor(
            &rebound_exports,
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(declaration_id)]),
        ),
        Err(UiOverlayCompositionDenial::ReconstructionRequired)
    ));
    let reconstructed = composition
        .reconstruct(&rebound_exports)
        .expect("rebind rebuilds from the new owner vector");
    assert_eq!(
        reconstructed.snapshot().generation(),
        &super::super::UiOverlayApplicationGeneration::from_prepared(new_generation)
    );
    assert_eq!(reconstructed.snapshot().presentation(), new_presentation);
    composition.retain_prepared(reconstructed).unwrap();
    assert_eq!(
        composition.current().unwrap().presentation(),
        new_presentation
    );
}

#[test]
fn owner_bridge_rematerializes_only_motion_dependents_on_successor() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(34).unwrap();
    let declaration_id = worth_ui_dsl::UiPortalDeclarationId::new(340).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let portal = owner_portal(37);
    let backdrop = declaration(
        35,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(declaration_id),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(declaration_id),
        worth_ui_dsl::UiBackdropMotionBasis::PortalPresentation(declaration_id),
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(declaration_id),
    );
    let mut portal_owner = owner_state();
    commit_open(&mut portal_owner, open_request(portal, runtime_surface, 35));
    let generation = prepared_generation();
    let extent = surface_extent(surface, runtime_surface, 2);
    let presentation = presentation();
    let binding = [UiOverlayPortalBinding::new(declaration_id, portal)];
    let mut composition = UiOverlayCompositionOwner::admit([backdrop], 3).unwrap();
    let initial_motion = motion_export(
        &generation,
        portal_owner.revision(),
        [UiOverlayMotionBinding::new(declaration_id, portal, 1)],
    );
    let initial = composition
        .prepare_initial(&owner_exports(
            &generation,
            &portal_owner,
            extent.clone(),
            presentation,
            binding,
            Some(initial_motion),
        ))
        .unwrap();
    composition.retain_prepared(initial).unwrap();
    let successor_motion = motion_export(
        &generation,
        portal_owner.revision() + 1,
        [UiOverlayMotionBinding::new(declaration_id, portal, 2)],
    );
    let successor = composition
        .prepare_successor(
            &owner_exports(
                &generation,
                &portal_owner,
                extent,
                presentation,
                binding,
                Some(successor_motion),
            ),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::PortalMotion(
                declaration_id,
            )]),
        )
        .expect("Motion successor rematerializes the local dependent");
    assert_eq!(
        successor.snapshot().motion_revision(),
        Some(portal_owner.revision() + 1)
    );
    assert_eq!(successor.counters().backdrop_declarations_selected(), 1);
}

#[test]
fn owner_bridge_rejects_incoherent_authoritative_exports_before_planning() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(35).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let owner = owner_state();
    let generation = prepared_generation();
    let binding_export = |generation| {
        UiOverlayPortalBindingExport::from_prepared(
            generation,
            runtime_surface,
            owner.revision(),
            [],
        )
        .unwrap()
    };
    let assemble = |generation, extent, bindings, motion| {
        UiOverlayOwnerExportVector::from_prepared(
            generation,
            UiOverlayPortalOwnerExport::from_owner(&owner),
            extent,
            presentation(),
            bindings,
            motion,
        )
    };
    assert_eq!(
        assemble(
            generation.clone(),
            surface_extent(surface, runtime_surface, 0),
            binding_export(generation.clone()),
            None,
        ),
        Err(UiOverlayOwnerExportDenial::InvalidExtentRevision)
    );
    assert_eq!(
        assemble(
            prepared_generation_variant(),
            surface_extent(surface, runtime_surface, 1),
            binding_export(generation.clone()),
            None,
        ),
        Err(UiOverlayOwnerExportDenial::BindingGenerationMismatch)
    );
    let foreign_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    assert_eq!(
        assemble(
            generation.clone(),
            surface_extent(surface, foreign_surface, 1),
            binding_export(generation.clone()),
            None,
        ),
        Err(UiOverlayOwnerExportDenial::BindingSurfaceMismatch)
    );
    let stale_binding = UiOverlayPortalBindingExport::from_prepared(
        generation.clone(),
        runtime_surface,
        owner.revision() + 1,
        [],
    )
    .unwrap();
    assert_eq!(
        assemble(
            generation.clone(),
            surface_extent(surface, runtime_surface, 1),
            stale_binding,
            None,
        ),
        Err(UiOverlayOwnerExportDenial::BindingPortalRevisionMismatch)
    );
    assert_eq!(
        assemble(
            generation.clone(),
            surface_extent(surface, runtime_surface, 1),
            binding_export(generation.clone()),
            Some(motion_export(
                &prepared_generation_variant(),
                owner.revision(),
                [],
            )),
        ),
        Err(UiOverlayOwnerExportDenial::MotionGenerationMismatch)
    );
}
