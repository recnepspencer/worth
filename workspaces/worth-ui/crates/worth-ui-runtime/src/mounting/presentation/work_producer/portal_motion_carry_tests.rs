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
    current
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
    successor.carry_portal_motion(&current, world.first_instance);
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
    successor.carry_portal_motion(&current, world.first_instance);
    assert_eq!(
        successor.motion_for_command(command),
        current.motion_for_command(command)
    );
    assert!(
        successor.carried_motion.is_empty(),
        "the host already shows a shared slot, so the frame restates nothing"
    );
}
