use super::*;
use crate::mounting::presentation::presented_surface_witness_for_certification;
use crate::mounting::presentation::work_producer_tests::world::{
    rect_spec, MountedPresentationWorld,
};
use crate::mounting::presentation::{
    motion_sampling::UiMountedMotionSampler, UiMountedPresentationLeaseGate,
};
use crate::runtime::motion::{UiMotionCommitReceipt, UiMotionDeclaration};
use worth_ui_host_contract::*;

/// A frame showing its rect through the rect's own Portal, with that Portal
/// layer accepted.
fn shown_through_portal(world: &MountedPresentationWorld) -> UiMountedPresentationState {
    shown_through_portal_with_a_tick_in_flight(world).0
}

/// A frame showing its rect through its Portal at tick 1, and the Portal
/// layers tick 2, issued but not yet landed, shows the Portal through.
fn shown_through_portal_with_a_tick_in_flight(
    world: &MountedPresentationWorld,
) -> (UiMountedPresentationState, UiIssuedCommandMotion) {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let projection = world.projection(frame, [rect_spec(world.first_instance, 0.0)]);
    let current = UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    let presentation = UiHostObservationPresentationBasis::new(
        world.requirement.host_surface(),
        frame,
        world.requirement.binding(),
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(UiMotionCommitReceipt::for_sampling_test_transition(
            840,
            UiMotionTargetIdentity::from_portal_owner(
                world.requirement.semantic_surface(),
                world.first_instance,
                world.first_instance.diagnostic_value(),
            ),
            presentation,
            None,
            true,
            None,
            false,
            UiMotionDeclaration::portal_exit(),
            None,
        ))
        .unwrap();
    let lease = UiMountedPresentationLeaseGate::default().claim().unwrap();
    let tick = sampler.prepare_tick(1, presentation).unwrap();
    let (_, acceptance) = current
        .prepare_motion_sample(tick.receipt(), presentation, &lease)
        .unwrap();
    acceptance
        .accept(
            &current,
            &presented_surface_witness_for_certification(presentation),
        )
        .unwrap();
    let in_flight = sampler.prepare_tick(2, presentation).unwrap();
    let (_, landing) = current
        .prepare_motion_sample(in_flight.receipt(), presentation, &lease)
        .unwrap();
    (current, landing.issued_motion())
}

fn rect_command(
    state: &UiMountedPresentationState,
    instance: UiMountedInstanceIdentity,
) -> UiMountedPaintCommandIdentity {
    state
        .commands_by_instance
        .get(&instance)
        .unwrap()
        .iter()
        .next()
        .unwrap()
        .identity()
}

#[test]
fn a_command_replaced_inside_a_portal_keeps_showing_through_the_portal_layer() {
    let world = MountedPresentationWorld::new();
    let current = shown_through_portal(&world);
    let command = rect_command(&current, world.first_instance);
    let shown = current.motion_for_command(command).flatten();
    assert!(shown.is_some(), "the Portal layer was accepted");
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let moved = world.projection(frame, [rect_spec(world.first_instance, 8.0)]);
    let mut successor =
        UiMountedPresentationState::from_projection(&moved, world.requirement, None);
    assert_eq!(rect_command(&successor, world.first_instance), command);
    assert_eq!(successor.motion_for_command(command), Some(None));
    successor.carry_portal_motion(&current, world.first_instance, &Default::default());
    assert_eq!(
        successor.motion_for_command(command).flatten(),
        shown,
        "the replaced command shows through the Portal's accepted sample"
    );
    let mut overrides = Vec::new();
    successor.add_carried_motion_overrides(&mut overrides);
    assert_eq!(
        overrides
            .iter()
            .map(|change| change.command())
            .collect::<Vec<_>>(),
        vec![command],
        "the host retired what it showed the old command through, so the frame restates it"
    );
}

#[test]
fn a_slot_shared_with_the_predecessor_is_never_written_by_the_carry() {
    let world = MountedPresentationWorld::new();
    let current = shown_through_portal(&world);
    let command = rect_command(&current, world.first_instance);
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let same = world.projection(frame, [rect_spec(world.first_instance, 0.0)]);
    let mut successor = UiMountedPresentationState::from_projection(&same, world.requirement, None);
    successor.inherit_unchanged_motion(&current, &[world.first_instance]);
    successor.carry_portal_motion(&current, world.first_instance, &Default::default());
    assert_eq!(
        successor.motion_for_command(command),
        current.motion_for_command(command)
    );
    assert!(
        successor.carried_motion.is_empty(),
        "the host already shows a shared slot, so the frame restates nothing"
    );
}

/// The host lands a tick issued before a frame first, so what the frame
/// carries stands with the rest of the Portal once both land.
#[test]
fn a_command_replaced_while_a_tick_is_in_flight_carries_the_ticks_portal_layer() {
    let world = MountedPresentationWorld::new();
    let (current, issued) = shown_through_portal_with_a_tick_in_flight(&world);
    let command = rect_command(&current, world.first_instance);
    let tick = |state: &UiMountedPresentationState| {
        state
            .motion_for_command(command)
            .flatten()
            .map(|sample| sample.tick())
    };
    assert_eq!(tick(&current), Some(1), "only tick 1 has landed");
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let moved = world.projection(frame, [rect_spec(world.first_instance, 8.0)]);
    let mut successor =
        UiMountedPresentationState::from_projection(&moved, world.requirement, None);
    successor.carry_portal_motion(&current, world.first_instance, &issued);
    assert_eq!(tick(&successor), Some(2));
    let mut overrides = Vec::new();
    successor.add_carried_motion_overrides(&mut overrides);
    assert_eq!(
        overrides
            .iter()
            .map(|change| change.command())
            .collect::<Vec<_>>(),
        vec![command],
    );
}

/// A frame issued behind a tick holds each command it shares with its
/// predecessor at the tick's layer in a slot of its own, and restates it, so
/// the predecessor keeps what the host showed before the tick.
#[test]
fn a_frame_issued_behind_a_tick_holds_each_shared_command_at_the_ticks_layer() {
    let world = MountedPresentationWorld::new();
    let (current, issued) = shown_through_portal_with_a_tick_in_flight(&world);
    let command = rect_command(&current, world.first_instance);
    let tick = |state: &UiMountedPresentationState| {
        state
            .motion_for_command(command)
            .flatten()
            .map(|sample| sample.tick())
    };
    let mut successor = current.clone();
    successor.hold_issued_motion(&issued);
    assert_eq!(tick(&successor), Some(2));
    assert_eq!(
        tick(&current),
        Some(1),
        "the predecessor keeps its own slot"
    );
    assert_eq!(successor.carried_motion, vec![command]);
}

/// A Scroll group's standing binds with each command's displayed base, so a
/// command the tick moves through a Scroll layer stays shared.
#[test]
fn a_command_a_tick_moves_through_a_scroll_layer_is_left_shared() {
    let world = MountedPresentationWorld::new();
    let (current, issued) = shown_through_portal_with_a_tick_in_flight(&world);
    let command = rect_command(&current, world.first_instance);
    let shared = current.motion_slot(command).unwrap().clone();
    let scrolled = UiIssuedCommandMotion::landing(issued.displayed().map(|mut displayed| {
        displayed.layers = displayed
            .layers
            .with_scroll(Some(super::super::command_motion_layers::tests::scroll()));
        (command, shared.clone(), displayed)
    }));
    let mut successor = current.clone();
    successor.hold_issued_motion(&scrolled);
    assert!(successor.motion_slot(command).unwrap().is(&shared));
    assert!(successor.carried_motion.is_empty());
}
