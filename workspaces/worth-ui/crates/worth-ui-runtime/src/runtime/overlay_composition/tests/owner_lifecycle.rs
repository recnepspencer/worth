use super::super::{
    UiOverlayChangeSet, UiOverlayChangedBasis, UiOverlayCompositionOwner, UiOverlayPortalBinding,
    UiOverlayStackParticipant,
};
use super::owner_support::{
    close_request, commit_open, owner_exports, owner_portal, owner_state, prepared_generation,
};
use super::support::{declaration, presentation, surface_extent};
use crate::runtime::portal::{UiPortalLifecyclePosture, UiPortalRuntimeState};

fn close(
    owner: &mut UiPortalRuntimeState,
    request: crate::runtime::portal::UiPortalServiceRequest,
) {
    let transition = owner.prepare(request).expect("owner close prepares");
    owner
        .commit_published(transition)
        .expect("owner close commits");
}

fn open(
    owner: &mut UiPortalRuntimeState,
    portal: crate::runtime::portal::UiPortalIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    lineage: u64,
) {
    commit_open(
        owner,
        super::owner_support::open_request(portal, surface, lineage),
    );
}

#[test]
fn owner_bridge_consumes_push_topmost_close_reconstruction_and_shutdown() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = owner_portal(1);
    let second = owner_portal(3);
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    );
    let extent = surface_extent(surface, runtime_surface, 2);
    let generation = prepared_generation();
    let mut owner = owner_state();
    open(&mut owner, first, runtime_surface, 1);
    open(&mut owner, second, runtime_surface, 2);
    assert_eq!(owner.active_count(), 2);
    assert_eq!(owner.stack_snapshot().owner_revision(), owner.revision());
    assert_eq!(owner.current_mounted_projection_inputs().len(), 2);
    let presentation = presentation();
    let bindings = [
        UiOverlayPortalBinding::new(portal_declaration, first),
        UiOverlayPortalBinding::new(portal_declaration, second),
    ];
    let mut composition = UiOverlayCompositionOwner::admit([backdrop], 3)
        .expect("production owner admits overlay state");
    let initial_exports = owner_exports(
        &generation,
        &owner,
        extent.clone(),
        presentation,
        bindings,
        None,
    );
    let initial = composition
        .prepare_initial(&initial_exports)
        .expect("owner bridge consumes the real Portal export");
    assert_eq!(initial.snapshot().portal_revision(), owner.revision());
    assert_eq!(
        initial
            .snapshot()
            .participants()
            .iter()
            .filter(|participant| { matches!(participant, UiOverlayStackParticipant::Portal(_)) })
            .count(),
        2
    );
    composition.retain_prepared(initial).unwrap();
    let reconstructed = composition
        .reconstruct(&owner_exports(
            &generation,
            &owner,
            extent.clone(),
            presentation,
            bindings,
            None,
        ))
        .expect("owner bridge reconstructs from sealed exports");
    assert_eq!(
        reconstructed.snapshot(),
        composition.current().expect("retained owner composition")
    );

    close(&mut owner, close_request(second, runtime_surface, 3));
    assert_eq!(owner.active_count(), 1);
    let retained_binding = [UiOverlayPortalBinding::new(portal_declaration, first)];
    let successor = composition
        .prepare_successor(
            &owner_exports(
                &generation,
                &owner,
                extent.clone(),
                presentation,
                retained_binding,
                None,
            ),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .expect("topmost close successor consumes owner export");
    assert_eq!(successor.reservation().portal_rows, 1);
    composition.retain_prepared(successor).unwrap();

    let shutdown = owner.shutdown();
    assert_eq!(shutdown.final_active_records(), 0);
    let terminal = composition
        .prepare_successor(
            &owner_exports(&generation, &owner, extent, presentation, [], None),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .expect("shutdown successor consumes empty owner export");
    assert!(terminal.snapshot().participants().is_empty());
}

#[test]
fn owner_bridge_retains_real_exit_until_terminal_owner_transition() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(2).unwrap();
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(20).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let runtime_portal = owner_portal(5);
    let backdrop = declaration(
        2,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    );
    let extent = surface_extent(surface, runtime_surface, 2);
    let generation = prepared_generation();
    let binding = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    let mut owner = owner_state();
    open(&mut owner, runtime_portal, runtime_surface, 10);
    let presentation = presentation();
    let mut composition = UiOverlayCompositionOwner::admit([backdrop], 3)
        .expect("production owner admits overlay state");
    let initial = composition
        .prepare_initial(&owner_exports(
            &generation,
            &owner,
            extent.clone(),
            presentation,
            binding,
            None,
        ))
        .unwrap();
    composition.retain_prepared(initial).unwrap();

    let close = owner
        .prepare(close_request(runtime_portal, runtime_surface, 11))
        .unwrap();
    let (_, retention) = owner
        .commit_published_with_exit_retention(close, true)
        .unwrap();
    let retention = retention.expect("owner issues exit retention");
    assert_eq!(
        owner.stack_snapshot().rows()[0].lifecycle(),
        UiPortalLifecyclePosture::Closing
    );
    let closing = composition
        .prepare_successor(
            &owner_exports(
                &generation,
                &owner,
                extent.clone(),
                presentation,
                binding,
                None,
            ),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .unwrap();
    assert!(closing.snapshot().participants().iter().any(|participant| {
        matches!(participant, UiOverlayStackParticipant::Portal(row) if row.lifecycle() == UiPortalLifecyclePosture::Closing)
    }));
    composition.retain_prepared(closing).unwrap();

    let terminal_transition = owner
        .prepare_exit_terminal(retention, super::owner_support::idempotency(12))
        .expect("owner retention prepares terminal exit");
    owner
        .commit_published_with_exit_retention(terminal_transition, false)
        .unwrap();
    let terminal = composition
        .prepare_successor(
            &owner_exports(&generation, &owner, extent, presentation, [], None),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .unwrap();
    assert!(terminal.snapshot().participants().is_empty());
}
