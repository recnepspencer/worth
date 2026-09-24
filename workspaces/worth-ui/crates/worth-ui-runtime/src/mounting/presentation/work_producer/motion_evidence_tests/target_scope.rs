use super::*;
use crate::mounting::presentation::presented_surface_witness_for_certification;

#[test]
fn ordinary_motion_cannot_overwrite_portal_paint_for_the_same_owner() {
    let world = MountedPresentationWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let projection = world.projection(frame, [rect_spec(world.first_instance, 0.0)]);
    let layout = crate::mounting::qualified_text_test_support::UiQualifiedTextTestFixture::new()
        .layout("WORTH");
    let text = world.text_projection(frame, &[world.first_instance], None, layout.view());
    let portal_command = projection.authored_paint_commands()[0].clone();
    let text_command = text.authored_paint_commands()[0].clone();
    let mut current =
        UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    current.commands_by_instance.insert(
        world.first_instance,
        UiMountedPresentationCommandBundle::from_commands(&[
            portal_command.clone(),
            text_command.clone(),
        ]),
    );
    let presentation = UiHostObservationPresentationBasis::new(
        world.requirement.host_surface(),
        frame,
        world.requirement.binding(),
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let key = world.first_instance.diagnostic_value();
    let surface = world.requirement.semantic_surface();
    let ordinary = UiMotionTargetIdentity::from_mounted_owner(surface, world.first_instance, key);
    let portal = UiMotionTargetIdentity::from_portal_owner(surface, world.first_instance, key);
    let lease = UiMountedPresentationLeaseGate::default().claim().unwrap();
    // Same owner and key; only the owner-issued scope distinguishes these targets.
    for (target, expected) in [
        (ordinary, text_command.identity()),
        (portal, portal_command.identity()),
    ] {
        let mut sampler = UiMountedMotionSampler::default();
        sampler
            .install(UiMotionCommitReceipt::for_sampling_test_transition(
                901,
                target,
                presentation,
                None,
                true,
                None,
                false,
                UiMotionDeclaration::portal_exit(),
                None,
            ))
            .unwrap();
        let initial = sampler.prepare_tick(1, presentation).unwrap();
        sampler.commit_prepared(initial.presented_for_certification());
        let tick = sampler.prepare_tick(200, presentation).unwrap();
        let before_portal = current.motion_for_command(portal_command.identity());
        let before_text = current.motion_for_command(text_command.identity());
        let (work, acceptance) = current
            .prepare_motion_sample(tick.receipt(), presentation, &lease)
            .unwrap();
        let UiMountedPresentationWorkView::Sample(sample) = work.view() else {
            panic!("sample work")
        };
        assert_eq!(sample.changes().len(), 1);
        assert_eq!(sample.changes()[0].command(), expected);
        assert_eq!(sample.changes()[0].opacity().units(), 0);
        acceptance
            .accept(
                &current,
                &presented_surface_witness_for_certification(presentation),
            )
            .unwrap();
        if target == ordinary {
            assert_eq!(
                current.motion_for_command(portal_command.identity()),
                before_portal
            );
        } else {
            assert_eq!(
                current.motion_for_command(text_command.identity()),
                before_text
            );
        }
        assert_eq!(
            current
                .motion_for_command(expected)
                .flatten()
                .unwrap()
                .opacity_units(),
            0
        );
    }
    let mut missing_group = UiMountedMotionSampler::default();
    missing_group
        .install(UiMotionCommitReceipt::for_sampling_test_transition(
            902,
            UiMotionTargetIdentity::from_portal_owner(surface, world.first_instance, key + 1),
            presentation,
            None,
            true,
            None,
            false,
            UiMotionDeclaration::portal_exit(),
            None,
        ))
        .unwrap();
    let tick = missing_group.prepare_tick(1, presentation).unwrap();
    assert!(matches!(
        current.prepare_motion_sample(tick.receipt(), presentation, &lease),
        Err(super::super::super::motion_sample::UiMountedMotionSampleWorkDenial::UnknownTargetCommands)
    ), "missing Portal membership must not fall back to its owner's ordinary commands");
}
