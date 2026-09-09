use super::*;
use crate::mounting::projection::appearance::{
    UiMountedAppearanceGeometryScope, UiMountedAppearanceSurfaceOverlayInput,
};
use crate::mounting::projection::frame_storage::appearance_state::UiMountedAppearanceFrameState;

#[test]
fn rebound_portal_recompletion_keeps_raw_and_completed_coordinate_ownership_current() {
    let mut world = GeometryWorld::new();
    let initial = world.frame(&[0, 1], None);
    let mut rebinding = initial.clone();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    world.bindings[0] = binding;
    rebinding
        .semantic
        .rebind_surface_allocations(world.surfaces[0], binding);
    rebinding
        .semantic
        .replace_surface(UiMountedProjectionSurface {
            surface: world.surfaces[0],
            binding,
            audience: UiMountedProjectionAudience::full(),
        });
    let rebound = world.frame(&[0, 1], Some(&rebinding));
    let child = world.children[0];
    assert_geometry(
        &context(&rebound, child),
        [8., 52., 220., 120.],
        [20_000, 60_000, 280_000, 320_000],
    );
    let node = rebound.semantic.node(child).unwrap();
    let UiMountedAllocationProjection::Known {
        basis: occurrence, ..
    } = node.occurrence_allocation
    else {
        panic!("raw occurrence is known");
    };
    let UiMountedAllocationProjection::Known {
        basis: completed, ..
    } = node.appearance_geometry.allocation
    else {
        panic!("completed Portal occurrence is known");
    };
    let UiMountedAllocationProjection::Known { basis: receipt, .. } = node.receipt.allocation()
    else {
        panic!("rebound receipt is known");
    };
    assert_eq!(occurrence, receipt);
    assert_eq!(completed, receipt);
    assert_eq!(
        rebound
            .semantic
            .node(world.children[1])
            .unwrap()
            .appearance_geometry,
        initial
            .semantic
            .node(world.children[1])
            .unwrap()
            .appearance_geometry
    );
}

// This is the retained mounted boundary: an authored launch is covered separately.
#[test]
fn attached_portal_cannot_use_structural_order_to_hide_missing_retained_appearance() {
    let mut world = GeometryWorld::new();
    let frame = world.frame(&[0, 1], None);
    let portal = world.owners[0];
    assert!(frame
        .portal_has_appearance_attachment(portal, world.surfaces[0])
        .unwrap());
    let binding = worth_ui_host_contract::UiMountedSurfaceBindingRequirement::new(
        world.surfaces[0],
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        world.bindings[0],
        WorthUiHostCapabilityObservationGeneration::new(1),
        1,
        worth_ui_host_contract::UiHostSurfacePresentationMode::RecordOnly,
    );
    let geometry = UiMountedAppearanceGeometryScope::new(&[binding], None);
    let overlay = UiMountedAppearanceSurfaceOverlayInput {
        semantic_surface: world.surfaces[0],
        portal_revision: 1,
        backdrop_revision: 1,
        portal_instances: Box::new([portal]),
        backdrops: Box::new([]),
        bottom_to_top: Box::new([
            worth_ui_host_contract::UiOverlayParticipantIdentity::Portal(portal),
        ]),
    };
    let mut missing = UiMountedAppearanceFrameState::default();
    let result = missing.lower_overlays(
        &frame,
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
        &geometry,
        &[overlay],
    );
    assert!(matches!(result, Err(crate::mounting::projection::frame_storage::appearance_output::UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable)));
    assert!(frame
        .portal_has_appearance_attachment(portal, world.surfaces[1])
        .is_err());
    assert!(frame
        .portal_has_appearance_attachment(
            UiMountedInstanceIdentity::mint_unbound().unwrap(),
            world.surfaces[0]
        )
        .is_err());
}

#[test]
fn portal_appearance_membership_is_local_to_requested_surface_and_deregistration() {
    let mut world = GeometryWorld::new();
    let frame = world.frame(&[0, 1], None);
    let bindings = [
        appearance_binding(world.surfaces[0], world.bindings[0]),
        appearance_binding(world.surfaces[1], world.bindings[1]),
    ];
    let overlays = [
        appearance_overlay(world.surfaces[0], world.owners[0]),
        appearance_overlay(world.surfaces[1], world.owners[1]),
    ];
    let mut state = UiMountedAppearanceFrameState::default();
    state
        .stage_portal_ownership_changes(&frame, &bindings, &overlays)
        .unwrap();
    state.retain_overlay_sidecar_for_test(world.surfaces[1]);

    state
        .stage_portal_ownership_changes(&frame, &bindings[..1], &overlays[..1])
        .unwrap();
    assert!(state.has_active_portal_for_test(world.surfaces[0], world.owners[0]));
    assert!(state.has_active_portal_for_test(world.surfaces[1], world.owners[1]));
    assert!(state.has_overlay_sidecar_for_test(world.surfaces[1]));

    let state = state.after_surface_deregistration(world.surfaces[1]);
    assert!(!state.has_active_portal_for_test(world.surfaces[1], world.owners[1]));
    assert!(!state.has_overlay_sidecar_for_test(world.surfaces[1]));
    assert!(state.has_active_portal_for_test(world.surfaces[0], world.owners[0]));
}

fn appearance_binding(
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) -> worth_ui_host_contract::UiMountedSurfaceBindingRequirement {
    worth_ui_host_contract::UiMountedSurfaceBindingRequirement::new(
        surface,
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        binding,
        WorthUiHostCapabilityObservationGeneration::new(1),
        1,
        worth_ui_host_contract::UiHostSurfacePresentationMode::RecordOnly,
    )
}

fn appearance_overlay(
    surface: UiSemanticSurfaceIdentity,
    portal: UiMountedInstanceIdentity,
) -> UiMountedAppearanceSurfaceOverlayInput {
    UiMountedAppearanceSurfaceOverlayInput {
        semantic_surface: surface,
        portal_revision: 1,
        backdrop_revision: 1,
        portal_instances: Box::new([portal]),
        backdrops: Box::new([]),
        bottom_to_top: Box::new([
            worth_ui_host_contract::UiOverlayParticipantIdentity::Portal(portal),
        ]),
    }
}
