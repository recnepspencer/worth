use super::super::{
    UiOverlayApplicationGeneration, UiOverlayChangeSet, UiOverlayChangedBasis,
    UiOverlayCompositionCoordinator, UiOverlayCompositionOwnerInput, UiOverlayCompositionState,
    UiOverlayPortalBinding, UiOverlayStackParticipant,
};
use super::support::{declaration, presentation, surface_extent};
use crate::runtime::interaction::{UiPresentedInteractionGeometry, UiPresentedViewportGeometry};
use crate::runtime::portal::{
    UiPortalDismissalCause, UiPortalIdentity, UiPortalLifecyclePosture, UiPortalRuntimeState,
    UiPortalServiceRequest,
};

fn owner_state() -> UiPortalRuntimeState {
    UiPortalRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
    )
}

fn owner_portal(graph: u64) -> UiPortalIdentity {
    UiPortalIdentity::for_test(graph)
}

fn idempotency(
    lineage: u64,
) -> crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity {
    crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, lineage)
}

fn presentation_geometry(epoch: u64) -> UiPresentedInteractionGeometry {
    let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound()
        .expect("test binding identity capacity");
    let presentation = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound()
            .expect("test host surface identity capacity"),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound()
            .expect("test frame identity capacity"),
        binding,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(epoch),
    );
    UiPresentedInteractionGeometry::for_test(presentation)
}

fn open_request(
    portal: UiPortalIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    lineage: u64,
) -> UiPortalServiceRequest {
    let geometry = presentation_geometry(lineage);
    UiPortalServiceRequest::open(
        portal,
        idempotency(lineage),
        geometry,
        Some(UiPresentedViewportGeometry::for_test(
            geometry.clip_bounds(),
            geometry.presentation(),
        )),
        surface,
    )
}

fn close_request(
    portal: UiPortalIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    lineage: u64,
) -> UiPortalServiceRequest {
    UiPortalServiceRequest::close(
        portal,
        idempotency(lineage),
        UiPortalDismissalCause::Escape,
        surface,
    )
}

fn open(
    owner: &mut UiPortalRuntimeState,
    portal: UiPortalIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    lineage: u64,
) {
    let transition = owner
        .prepare(open_request(portal, surface, lineage))
        .expect("owner open prepares");
    owner
        .commit_published(transition)
        .expect("owner open commits");
}

fn owner_input<'a>(
    generation: u64,
    extent: &'a super::super::UiOverlaySurfaceExtentSnapshot,
    owner: &'a UiPortalRuntimeState,
    bindings: &'a [UiOverlayPortalBinding],
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
) -> UiOverlayCompositionOwnerInput<'a> {
    UiOverlayCompositionOwnerInput::new(
        UiOverlayApplicationGeneration::for_test(generation),
        presentation,
        extent,
        owner,
        bindings,
        None,
    )
}

fn coordinator(backdrop: worth_ui_dsl::UiBackdropDeclaration) -> UiOverlayCompositionCoordinator {
    UiOverlayCompositionCoordinator::new(
        UiOverlayCompositionState::admit([backdrop], 3, Default::default())
            .expect("admitted overlay state"),
    )
}

#[test]
fn coordinator_consumes_owner_push_topmost_close_and_shutdown_exports() {
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
    let mut coordinator = coordinator(backdrop);
    let initial = coordinator
        .prepare_initial(&owner_input(1, &extent, &owner, &bindings, presentation))
        .expect("coordinator consumes the real owner stack");
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
    coordinator.retain_prepared(initial).unwrap();
    let reconstructed = coordinator
        .reconstruct(&owner_input(1, &extent, &owner, &bindings, presentation))
        .expect("coordinator reconstructs from the owner export");
    assert_eq!(
        reconstructed.snapshot(),
        coordinator.current().expect("published owner composition")
    );

    let close = owner
        .prepare(close_request(second, runtime_surface, 3))
        .expect("topmost close prepares");
    owner
        .commit_published(close)
        .expect("topmost close commits");
    assert_eq!(owner.active_count(), 1);
    assert_eq!(owner.stack_snapshot().rows().len(), 1);
    let retained_binding = [UiOverlayPortalBinding::new(portal_declaration, first)];
    let successor = coordinator
        .prepare_successor(
            &owner_input(1, &extent, &owner, &retained_binding, presentation),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .expect("close successor consumes owner export");
    assert_eq!(successor.snapshot().portal_revision(), owner.revision());
    assert_eq!(successor.reservation().portal_rows, 1);
    coordinator.retain_prepared(successor).unwrap();

    let shutdown = owner.shutdown();
    assert_eq!(shutdown.final_active_records(), 0);
    assert!(owner.stack_snapshot().rows().is_empty());
    let terminal = coordinator
        .prepare_successor(
            &owner_input(1, &extent, &owner, &[], presentation),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .expect("shutdown successor consumes the empty owner export");
    assert!(terminal.snapshot().participants().is_empty());
}

#[test]
fn coordinator_consumes_real_exit_retention_before_terminal_owner_removal() {
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
    let binding = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    let mut owner = owner_state();
    open(&mut owner, runtime_portal, runtime_surface, 10);
    let presentation = presentation();
    let mut coordinator = coordinator(backdrop);
    coordinator
        .retain_prepared(
            coordinator
                .prepare_initial(&owner_input(1, &extent, &owner, &binding, presentation))
                .unwrap(),
        )
        .unwrap();

    let close = owner
        .prepare(close_request(runtime_portal, runtime_surface, 11))
        .unwrap();
    let (_, retention) = owner
        .commit_published_with_exit_retention(close, true)
        .unwrap();
    let retention = retention.expect("owner issues exit retention");
    let closing_snapshot = owner.stack_snapshot();
    assert_eq!(
        closing_snapshot.rows()[0].lifecycle(),
        UiPortalLifecyclePosture::Closing
    );
    assert_eq!(owner.current_mounted_projection_inputs().len(), 1);
    let closing = coordinator
        .prepare_successor(
            &owner_input(1, &extent, &owner, &binding, presentation),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .unwrap();
    assert!(closing.snapshot().participants().iter().any(|participant| {
        matches!(participant, UiOverlayStackParticipant::Portal(row) if row.lifecycle() == UiPortalLifecyclePosture::Closing)
    }));
    coordinator.retain_prepared(closing).unwrap();

    let terminal_transition = owner
        .prepare_exit_terminal(retention, idempotency(12))
        .expect("exact owner retention prepares terminal exit");
    assert!(owner
        .mounted_projection_inputs(&terminal_transition, false)
        .is_empty());
    owner
        .commit_published_with_exit_retention(terminal_transition, false)
        .unwrap();
    assert!(owner.stack_snapshot().rows().is_empty());
    let terminal = coordinator
        .prepare_successor(
            &owner_input(1, &extent, &owner, &[], presentation),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .unwrap();
    assert!(terminal.snapshot().participants().is_empty());
}
