#[path = "motion_evidence_tests/empty_logical_damage.rs"]
mod empty_logical_damage;
#[path = "motion_evidence_tests/text_ambiguity.rs"]
mod text_ambiguity;

use super::*;
use crate::mounting::presentation::work_producer::command_bundle::UiMountedPresentationCommandBundle;
use crate::mounting::presentation::work_producer_tests::world::{
    rect_spec, MountedPresentationWorld,
};
use crate::mounting::presentation::{
    motion_sampling::UiMountedMotionSampler, UiMountedPresentationLeaseGate,
};
use crate::runtime::motion::{UiMotionCommitReceipt, UiMotionDeclaration, UiMotionTargetIdentity};
use worth_ui_host_contract::*;

#[test]
fn exact_command_slots_share_only_unchanged_overrides_and_accept_atomically() {
    // Mounted command fixture: two physical families belong to one instance.
    // This deliberately isolates command replacement from semantic authoring.
    let world = MountedPresentationWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let projection = world.projection(frame, [rect_spec(world.first_instance, 0.0)]);
    let layout = crate::mounting::qualified_text_test_support::UiQualifiedTextTestFixture::new()
        .layout("WORTH");
    let text = world.text_projection(frame, &[world.first_instance], None, layout.view());
    let text_command = text.authored_paint_commands()[0].clone();
    let rect_command = projection.authored_paint_commands()[0].clone();
    let commands = [text_command.clone(), rect_command.clone()];
    let mut current =
        UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    current.commands_by_instance.insert(
        world.first_instance,
        UiMountedPresentationCommandBundle::from_commands(&commands),
    );
    let presentation = UiHostObservationPresentationBasis::new(
        world.requirement.host_surface(),
        frame,
        world.requirement.binding(),
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(UiMotionCommitReceipt::for_sampling_test_transition(
            831,
            UiMotionTargetIdentity::from_family_owner(
                world.requirement.semantic_surface(),
                world.first_instance,
                831,
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
    let (work, acceptance) = current
        .prepare_motion_sample(tick.receipt(), presentation, &lease)
        .unwrap();
    let UiMountedPresentationWorkView::Sample(sample_work) = work.view() else {
        panic!("sample work")
    };
    assert_eq!(sample_work.changes().len(), 2);
    assert!(
        !sample_work.damage().is_empty(),
        "opacity-only work damages command coverage"
    );
    assert_eq!(
        current.motion_for_command(text_command.identity()),
        Some(None)
    );
    let mut older_candidate =
        UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    older_candidate.commands_by_instance.insert(
        world.first_instance,
        UiMountedPresentationCommandBundle::from_commands(&commands),
    );
    older_candidate.inherit_unchanged_motion(&current, &[world.first_instance]);
    acceptance.accept(&current, presentation).unwrap();
    sampler.commit_prepared(tick);
    for command in &commands {
        assert_eq!(
            older_candidate.motion_for_command(command.identity()),
            current.motion_for_command(command.identity())
        );
        assert_eq!(
            current
                .motion_for_command(command.identity())
                .flatten()
                .unwrap()
                .opacity_units(),
            65_535
        );
    }
    let mut reconstructed =
        UiMountedPresentationState::from_projection(&projection, world.requirement, Some(frame));
    reconstructed.commands_by_instance.insert(
        world.first_instance,
        UiMountedPresentationCommandBundle::from_commands(&commands),
    );
    reconstructed.inherit_reconstruction_motion(&current);
    assert_eq!(
        reconstructed.reconstruction_motion_overrides().len(),
        1,
        "the fixture's authored projection order contains its rect; the injected text still proves slot inheritance below"
    );
    for command in &commands {
        assert_eq!(
            reconstructed.motion_for_command(command.identity()),
            current.motion_for_command(command.identity()),
            "cold reconstruction shares accepted evidence only through exact command meaning"
        );
    }
    let successor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let successor_text = world.text_projection(
        successor_frame,
        &[world.first_instance],
        None,
        layout.view(),
    );
    let successor_rect = world.projection(successor_frame, [rect_spec(world.first_instance, 0.0)]);
    let mut advanced = UiMountedPresentationCommandBundle::from_commands(&[
        successor_text.authored_paint_commands()[0].clone(),
        successor_rect.authored_paint_commands()[0].clone(),
    ]);
    advanced.inherit_unchanged_motion(
        current
            .commands_by_instance
            .get(&world.first_instance)
            .unwrap(),
    );
    for command in &commands {
        assert_eq!(
            advanced.motion_slot(command.identity()).unwrap().sample(),
            current.motion_for_command(command.identity()).flatten(),
            "real successor frame and receipts preserve unchanged physical evidence"
        );
    }
    let changed_text = world.text_projection(
        successor_frame,
        &[world.first_instance],
        Some(0),
        layout.view(),
    );
    assert!(advanced.replace(changed_text.authored_paint_commands()[0].clone()));
    assert!(
        advanced
            .motion_slot(text_command.identity())
            .unwrap()
            .sample()
            .is_none(),
        "changing a legacy text foreground replaces the command's accepted evidence"
    );
    let moved = world.projection(successor_frame, [rect_spec(world.first_instance, 8.0)]);
    assert!(advanced.replace(moved.authored_paint_commands()[0].clone()));
    assert!(
        advanced
            .motion_slot(rect_command.identity())
            .unwrap()
            .sample()
            .is_none(),
        "geometry replacement cannot inherit an old override"
    );

    let tick = sampler.prepare_tick(56, presentation).unwrap();
    let (_, late) = current
        .prepare_motion_sample(tick.receipt(), presentation, &lease)
        .unwrap();
    let mut changed = rect_spec(world.first_instance, 0.0);
    changed.color = UiMountedRgba8::new(1, 2, 3, 255);
    let changed_projection = world.projection(frame, [changed]);
    let replacement = changed_projection.authored_paint_commands()[0].clone();
    assert_eq!(replacement.identity(), rect_command.identity());
    let mut bundle = current
        .commands_by_instance
        .get(&world.first_instance)
        .unwrap()
        .clone();
    assert!(bundle.replace(replacement));
    current
        .commands_by_instance
        .insert(world.first_instance, bundle);
    assert_eq!(
        current.motion_for_command(rect_command.identity()),
        Some(None)
    );
    let previous_text = current
        .motion_for_command(text_command.identity())
        .flatten()
        .unwrap();
    assert!(matches!(
        late.accept(&current, presentation),
        Err(UiCommandMotionAcceptanceDenial::CommandReplaced)
    ));
    assert_eq!(current.motion_for_command(text_command.identity()).flatten(), Some(previous_text),
        "later replaced command denies the whole acceptance before the first unchanged text slot writes");
    let reconstructed =
        UiMountedPresentationState::from_projection(&projection, world.requirement, Some(frame));
    assert_eq!(
        reconstructed.motion_for_command(rect_command.identity()),
        Some(None)
    );
}
