use super::{geometry, session::World};
use worth_ui_host_contract::*;

pub(super) fn appearance_changes(world: &mut World) {
    let presentations = world.surfaces.map(|surface| {
        world
            .session
            .mounted
            .current_presentation_for_surface(surface)
    });
    hover(world, 0, 1);
    let rejected = selected(world, &[world.instances[0]], 0);
    for _ in rejected.surfaces() {
        world.host.push_rejected();
    }
    assert!(matches!(
        world.session.present_prepared_mounted_frame_internal(
            rejected,
            UiPresentationDeadline::at_tick(u64::MAX),
            2
        ),
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    assert_eq!(
        world.surfaces.map(|surface| world
            .session
            .mounted
            .current_presentation_for_surface(surface)),
        presentations
    );
    let retried = selected(world, &[world.instances[0]], 0);
    world.publish(retried, 3, false);

    // The same pointer moves between the two mounted neighborhoods. Selection
    // must include departure and arrival even if an equal output hid one in pixels.
    hover(world, 1, 2);
    let changed = selected(world, &[world.instances[0], world.instances[1]], 1);
    world.publish(changed, 4, false);
    hover(world, 1, 3);
    let unchanged = world.prepare_surface(world.surfaces[0]);
    assert_eq!(
        unchanged
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0
    );
    unchanged.assert_no_unpublished_appearance_for_test();
}

fn selected(
    world: &mut World,
    expected: &[UiMountedInstanceIdentity],
    hovered: usize,
) -> crate::mounting::UiPreparedMountedFrame {
    let frame = world.prepare_surface(world.surfaces[0]);
    let invalidation = frame.appearance_invalidation_batch().unwrap();
    for target in expected {
        let graph = world
            .session
            .mounted
            .current_mounted_identity_basis(*target)
            .unwrap()
            .graph_node_identity();
        assert!(
            invalidation.requires_semantic_resolution(graph, *target),
            "owner-state target {target:?} retains semantic cause after batching"
        );
    }
    let cost = frame.appearance_selection_cost_report();
    assert_eq!(cost.selected_instance_count(), expected.len());
    assert_eq!(cost.index_entries_touched(), expected.len());
    let profile = worth_ui_host_native::appearance_capability_report();
    let output =
        frame.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
    let mut observed = Vec::new();
    let mut pointers = Vec::new();
    for fragment in output.fragments() {
        match fragment.identity() {
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor: Some(_),
                successor: Some(receipt),
            } => {
                observed.push(receipt.mounted_instance());
                assert_eq!(
                    fragment.work().posture(),
                    UiMountedAppearanceWorkPosture::Delta
                );
                assert_eq!(fragment.work().changes().len(), 1);
                assert!(matches!(
                    &fragment.work().changes()[0],
                    UiMountedAppearanceMechanicChange::Replace {
                        successor: UiMountedAppearanceMechanic::Surface(_),
                        ..
                    }
                ));
                assert!(fragment
                    .text_candidates()
                    .iter()
                    .all(|candidate| candidate.performed_layout_cost().is_none()));
            }
            UiUnpublishedAppearanceFragmentIdentity::SurfacePointer { surface, pointer } => {
                let [UiMountedAppearanceMechanic::Pointer(mechanic)] =
                    fragment.work().successor().mechanics()
                else {
                    panic!("one independent pointer mechanic");
                };
                assert_eq!(mechanic.target(), world.instances[hovered]);
                assert_eq!(mechanic.surface(), world.surfaces[0]);
                pointers.push((surface, pointer));
            }
            other => panic!("unrelated shared world output {other:?}"),
        }
    }
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    observed.sort_unstable();
    assert_eq!(observed, expected);
    assert_eq!(
        pointers,
        [(world.surfaces[0], UiHostPointerIdentity::new(1))]
    );
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(&output).unwrap();
    frame
}

pub(super) fn hover(world: &mut World, index: usize, sequence: u64) {
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol");
    };
    let sequence = UiHostObservationSequence::new(sequence);
    let [x, y, width, height] = geometry::BOXES[index];
    let report = UiHostObservationReport::new(
        sequence,
        UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
        UiHostObservationPayload::PointerMotion {
            pointer: UiHostPointerIdentity::new(1),
            capture_epoch: UiHostPointerCaptureEpoch::new(1),
            position: UiHostSurfacePosition::viewport_logical(
                ((x + width / 2.0) * 1_000.0) as i64,
                ((y + height / 2.0) * 1_000.0) as i64,
            ),
            pressed_buttons: UiHostPressedPointerButtons::NONE,
        },
    )
    .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
    .unwrap();
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: world.session.host_session.identity().as_u64(),
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![report],
    })
    .unwrap();
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) =
        world.session.admit_host_interaction_batch(batch)
    else {
        panic!("real shared pointer ingress must apply");
    };
    assert!(receipt.pointer_presence_denials().is_empty());
    if !receipt.pointer_presence_transitions().is_empty() {
        let mut turn = world.session.begin_observation_turn().unwrap();
        for transition in receipt.pointer_presence_transitions() {
            turn.admit_pointer_presence_transition(transition.clone())
                .unwrap();
        }
        let observations = turn.seal().unwrap();
        world.session.classify_observations(observations).unwrap();
    }
}
