use super::super::*;
use super::support::{
    box_at, declaration, input, input_with_generation, portal, portal_snapshot,
    portal_snapshot_with_rows, presentation, surface_extent, surface_extent_with_viewport,
};

fn backdrop_for(
    snapshot: &UiOverlayStackSnapshot,
    declaration: u64,
    portal: crate::runtime::portal::UiPortalIdentity,
) -> &UiOverlayBackdropRow {
    snapshot
        .participants()
        .iter()
        .find_map(|participant| match participant {
            UiOverlayStackParticipant::Backdrop(row)
                if row.declaration().value() == declaration
                    && row.identity().scope() == UiOverlayBackdropInstanceScope::Portal(portal) =>
            {
                Some(row)
            }
            _ => None,
        })
        .expect("expected portal-scoped backdrop")
}

#[test]
fn owner_portal_push_is_a_successor_over_a_published_empty_snapshot() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let runtime_portal = portal(30);
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    );
    let extent = surface_extent(surface, runtime_surface, 2);
    let empty = portal_snapshot(runtime_surface, []);
    let pushed = portal_snapshot(runtime_surface, [(runtime_portal, 1)]);
    let bindings = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    let presentation = presentation();
    let mut state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();
    state
        .publish(
            state
                .prepare_initial(input(&extent, &empty, &[], None, presentation))
                .unwrap(),
        )
        .unwrap();

    let successor = state
        .prepare_successor(
            input(&extent, &pushed, &bindings, None, presentation),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .unwrap();
    assert_eq!(successor.counters().backdrop_declarations_selected(), 1);
    assert!(successor
        .snapshot()
        .participants()
        .iter()
        .any(|participant| matches!(participant, UiOverlayStackParticipant::Portal(row) if row.portal() == runtime_portal)));
    assert_eq!(successor.reservation().backdrop_rows, 1);
}

#[test]
fn topmost_close_recomputes_only_the_closed_portal_neighborhood() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let first_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let second_declaration = worth_ui_dsl::UiPortalDeclarationId::new(11).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = portal(31);
    let second = portal(32);
    let first_backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(first_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(first_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(first_declaration),
    );
    let second_backdrop = declaration(
        2,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(second_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(second_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(second_declaration),
    );
    let extent = surface_extent(surface, runtime_surface, 2);
    let before = portal_snapshot(runtime_surface, [(first, 1), (second, 2)]);
    let after = portal_snapshot(runtime_surface, [(first, 1)]);
    let bindings = [
        UiOverlayPortalBinding::new(first_declaration, first),
        UiOverlayPortalBinding::new(second_declaration, second),
    ];
    let retained_binding = [UiOverlayPortalBinding::new(first_declaration, first)];
    let presentation = presentation();
    let mut state =
        UiOverlayCompositionState::admit([first_backdrop, second_backdrop], 3, Default::default())
            .unwrap();
    state
        .publish(
            state
                .prepare_initial(input(&extent, &before, &bindings, None, presentation))
                .unwrap(),
        )
        .unwrap();
    let previous = state.current().unwrap().clone();
    let successor = state
        .prepare_successor(
            input(&extent, &after, &retained_binding, None, presentation),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::PortalPresence(
                second_declaration,
            )]),
        )
        .unwrap();

    assert_eq!(successor.counters().backdrop_declarations_selected(), 1);
    assert_eq!(
        backdrop_for(successor.snapshot(), 1, first),
        backdrop_for(&previous, 1, first)
    );
    assert!(!successor
        .snapshot()
        .participants()
        .iter()
        .any(|participant| participant.identity() == UiOverlayParticipantIdentity::Portal(second)));
    assert!(!successor
        .snapshot()
        .participants()
        .iter()
        .any(|participant| matches!(participant, UiOverlayStackParticipant::Backdrop(row) if row.declaration().value() == 2)));
}

#[test]
fn parent_close_consumes_the_owner_closed_descendant_projection() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let parent_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let child_declaration = worth_ui_dsl::UiPortalDeclarationId::new(11).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let parent = portal(33);
    let child = portal(34);
    let declarations = [
        declaration(
            1,
            surface,
            worth_ui_dsl::UiBackdropScope::PerPortalInstance(parent_declaration),
            worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(parent_declaration),
            worth_ui_dsl::UiBackdropMotionBasis::None,
            worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(parent_declaration),
        ),
        declaration(
            2,
            surface,
            worth_ui_dsl::UiBackdropScope::PerPortalInstance(child_declaration),
            worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(child_declaration),
            worth_ui_dsl::UiBackdropMotionBasis::None,
            worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(child_declaration),
        ),
    ];
    let extent = surface_extent(surface, runtime_surface, 2);
    let before = portal_snapshot_with_rows(
        7,
        [
            (
                parent,
                None,
                runtime_surface,
                1,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            ),
            (
                child,
                Some(parent),
                runtime_surface,
                2,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            ),
        ],
    );
    let after = portal_snapshot(runtime_surface, []);
    let bindings = [
        UiOverlayPortalBinding::new(parent_declaration, parent),
        UiOverlayPortalBinding::new(child_declaration, child),
    ];
    let mut state = UiOverlayCompositionState::admit(declarations, 3, Default::default()).unwrap();
    let presentation = presentation();
    state
        .publish(
            state
                .prepare_initial(input(&extent, &before, &bindings, None, presentation))
                .unwrap(),
        )
        .unwrap();
    let successor = state
        .prepare_successor(
            input(&extent, &after, &[], None, presentation),
            &UiOverlayChangeSet::from_changes([
                UiOverlayChangedBasis::Portal(parent_declaration),
                UiOverlayChangedBasis::Portal(child_declaration),
            ]),
        )
        .unwrap();

    assert_eq!(successor.counters().backdrop_declarations_selected(), 2);
    assert!(successor.snapshot().participants().is_empty());
}

#[test]
fn exit_retention_keeps_closing_owner_rows_then_removes_them_at_terminal_exit() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(10).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let runtime_portal = portal(35);
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    );
    let extent = surface_extent(surface, runtime_surface, 2);
    let visible = portal_snapshot(runtime_surface, [(runtime_portal, 1)]);
    let closing = portal_snapshot_with_rows(
        8,
        [(
            runtime_portal,
            None,
            runtime_surface,
            1,
            crate::runtime::portal::UiPortalLifecyclePosture::Closing,
        )],
    );
    let terminal = portal_snapshot(runtime_surface, []);
    let binding = [UiOverlayPortalBinding::new(
        portal_declaration,
        runtime_portal,
    )];
    let presentation = presentation();
    let mut state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();
    state
        .publish(
            state
                .prepare_initial(input(&extent, &visible, &binding, None, presentation))
                .unwrap(),
        )
        .unwrap();
    let closing_successor = state
        .prepare_successor(
            input(&extent, &closing, &binding, None, presentation),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .unwrap();
    assert!(closing_successor.snapshot().participants().iter().any(
        |participant| matches!(participant, UiOverlayStackParticipant::Portal(row) if row.lifecycle() == crate::runtime::portal::UiPortalLifecyclePosture::Closing)
    ));
    state.publish(closing_successor).unwrap();
    let terminal_successor = state
        .prepare_successor(
            input(&extent, &terminal, &[], None, presentation),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(portal_declaration)]),
        )
        .unwrap();
    assert!(terminal_successor.snapshot().participants().is_empty());
}

#[test]
fn resize_recomputes_extent_dependents_without_mutating_the_predecessor() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let old_extent = surface_extent(surface, runtime_surface, 2);
    let new_extent = surface_extent_with_viewport(
        surface,
        runtime_surface,
        3,
        box_at(0.0, 0.0, 320.0, 240.0),
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
    let presentation = presentation();
    let mut state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();
    state
        .publish(
            state
                .prepare_initial(input(&old_extent, &portals, &[], None, presentation))
                .unwrap(),
        )
        .unwrap();
    let previous = state.current().unwrap().clone();
    let successor = state
        .prepare_successor(
            input(&new_extent, &portals, &[], None, presentation),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::SurfaceExtent(surface)]),
        )
        .unwrap();
    let UiOverlayStackParticipant::Backdrop(row) = &successor.snapshot().participants()[0] else {
        panic!("expected resized backdrop")
    };
    assert_eq!(row.extent().bounds(), box_at(0.0, 0.0, 320.0, 240.0));
    assert_eq!(state.current(), Some(&previous));
}

#[test]
fn rebind_requires_reconstruction_and_accepts_a_new_owner_generation() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let old_runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let new_runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let old_extent = surface_extent(surface, old_runtime_surface, 2);
    let new_extent = surface_extent(surface, new_runtime_surface, 4);
    let old_portals = portal_snapshot(old_runtime_surface, []);
    let new_portals = portal_snapshot(new_runtime_surface, []);
    let backdrop = declaration(
        1,
        surface,
        worth_ui_dsl::UiBackdropScope::SurfaceSingleton,
        worth_ui_dsl::UiBackdropPresenceBasis::Always,
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::AboveSurfaceContent,
    );
    let presentation = presentation();
    let mut state = UiOverlayCompositionState::admit([backdrop], 3, Default::default()).unwrap();
    state
        .publish(
            state
                .prepare_initial(input(&old_extent, &old_portals, &[], None, presentation))
                .unwrap(),
        )
        .unwrap();
    let previous = state.current().unwrap().clone();
    let changes = UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::SurfaceExtent(surface)]);
    assert_eq!(
        state.prepare_successor(
            input_with_generation(2, &new_extent, &new_portals, &[], None, presentation),
            &changes,
        ),
        Err(UiOverlayCompositionDenial::ReconstructionRequired)
    );
    assert_eq!(state.current(), Some(&previous));
    let reconstructed = state
        .reconstruct(input_with_generation(
            2,
            &new_extent,
            &new_portals,
            &[],
            None,
            presentation,
        ))
        .unwrap();
    assert_eq!(
        reconstructed.snapshot().runtime_surface(),
        new_runtime_surface
    );
    assert_eq!(
        reconstructed.snapshot().generation(),
        &UiOverlayApplicationGeneration::Test(2)
    );
}

#[test]
fn shutdown_empty_portal_snapshot_is_a_sealed_reconstructible_candidate() {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let extent = surface_extent(surface, runtime_surface, 2);
    let portals = portal_snapshot(runtime_surface, []);
    let state = UiOverlayCompositionState::admit([], 3, Default::default()).unwrap();

    let candidate = state
        .prepare_initial(input(&extent, &portals, &[], None, presentation()))
        .unwrap();
    assert!(candidate.snapshot().participants().is_empty());
    assert_eq!(candidate.reservation().portal_rows, 0);
    assert_eq!(candidate.reservation().backdrop_rows, 0);
}
