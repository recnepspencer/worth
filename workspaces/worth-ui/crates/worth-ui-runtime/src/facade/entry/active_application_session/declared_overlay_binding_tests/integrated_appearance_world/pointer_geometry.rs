use super::World;
use worth_ui_host_contract::*;

#[test]
fn queued_pointer_keeps_event_provenance_but_admits_hover_on_the_current_frame() {
    let mut world = World::launch();
    let frame = world.prepare();
    world.publish(frame, 1, true);
    let observed = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    // The fixed allocation places (150,55) in A on both presentations.
    let queued = pointer_batch(
        world.session.host_session.identity().as_u64(),
        observed,
        1,
        [150_000, 55_000],
    );
    let frame = world.prepare();
    world.publish(frame, 2, false);
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    assert_ne!(observed.frame(), current.frame());
    let event_target = crate::runtime::interaction::targeting::resolve_presented_target(
        &world.session.mounted,
        observed,
        UiHostSurfacePosition::viewport_logical(150_000, 55_000),
        &mut Default::default(),
    )
    .unwrap();
    assert_eq!(event_target.mounted_instance(), world.instances[0]);
    assert_eq!(event_target.presentation(), observed);
    world.host.enqueue_observation_for_next_drain(queued);
    let outcomes = world
        .session
        .drain_and_admit_host_observation_batches(Default::default())
        .into_outcomes();
    let [crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt)] =
        outcomes.as_ref()
    else {
        panic!("queued event must remain admissible: {outcomes:?}")
    };
    assert!(receipt.pointer_presence_denials().is_empty());
    let snapshot = world
        .session
        .interaction
        .pointer_presence_appearance_snapshot()
        .unwrap();
    let posture = snapshot
        .postures()
        .iter()
        .find(|row| row.pointer() == UiHostPointerIdentity::new(1))
        .unwrap();
    assert_eq!(posture.target(), Some(world.instances[0]));
    assert_eq!(posture.presentation(), current);
    assert_ne!(
        posture.presented_target().unwrap().node_receipt(),
        event_target.node_receipt()
    );
    let _ = world.session.shutdown();
    assert_eq!(world.host.pending_observation_batch_count(), 0);
}

#[test]
fn admitted_pointer_targets_the_painted_occurrence_on_each_surface() {
    let mut world = World::launch();
    for _ in world.surfaces {
        world.host.push_native_display_presented();
    }
    let request = world.session.mounted_frame_request();
    let outcome = world
        .session
        .execute_mounted_frame(
            request,
            UiPresentationDeadline::at_tick(u64::MAX),
            1,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("ordinary mounted publication must prepare"));
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    drop(outcome);

    // Independently chosen client points: unequal A rectangles and another
    // occurrence of A's declaration on B. No point is derived from a hit row.
    for (sequence, surface, instance, point, bounds) in [
        (1, 0, 0, [44_000, 54_000], [40.0, 50.0, 180.0, 60.0]),
        (2, 0, 1, [310_000, 55_000], [300.0, 50.0, 220.0, 70.0]),
        (3, 0, 2, [85_000, 205_000], [80.0, 200.0, 160.0, 60.0]),
        (4, 1, 3, [55_000, 75_000], [40.0, 50.0, 180.0, 60.0]),
    ] {
        let presentation = world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[surface])
            .unwrap();
        let batch = pointer_batch(
            world.session.host_session.identity().as_u64(),
            presentation,
            sequence,
            point,
        );
        let ingress = world.session.admit_host_interaction_batch(batch);
        let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = ingress
        else {
            panic!("pointer admission must reach targeting: {ingress:?}");
        };
        assert!(receipt.pointer_presence_denials().is_empty(), "{receipt:?}");
        let snapshot = world
            .session
            .interaction
            .pointer_presence_appearance_snapshot()
            .unwrap();
        let posture = snapshot
            .postures()
            .iter()
            .find(|row| row.pointer() == UiHostPointerIdentity::new(sequence))
            .unwrap();
        assert_eq!(posture.target(), Some(world.instances[instance]));
        assert_eq!(posture.presentation(), presentation);
        let target = posture.presented_target().unwrap();
        assert_eq!(target.binding(), presentation.binding());
        assert_eq!(target.presentation(), presentation);
        let actual = target.geometry().bounds();
        assert_eq!(
            actual.coordinate_space(),
            UiMountedCoordinateSpace::Viewport
        );
        assert_eq!(
            [actual.x(), actual.y(), actual.width(), actual.height()],
            bounds
        );

        let projection = world
            .session
            .mounted
            .current_projection_rc_for_test()
            .unwrap();
        let view = projection.view_for(presentation.binding()).unwrap();
        assert!(view
            .semantic_text()
            .rows()
            .iter()
            .any(|row| row.mounted_instance() == world.instances[instance] && row.text() == "AB"));
        for row in view
            .semantic_text()
            .rows()
            .iter()
            .filter(|row| row.mounted_instance() == world.instances[instance])
        {
            assert_eq!(
                row.bounds().coordinate_space(),
                UiMountedCoordinateSpace::Viewport
            );
            assert_eq!(
                row.clip_bounds().coordinate_space(),
                UiMountedCoordinateSpace::Viewport
            );
            let text_bounds = row.bounds();
            assert_eq!(
                [
                    text_bounds.x(),
                    text_bounds.y(),
                    text_bounds.width(),
                    text_bounds.height()
                ],
                bounds
            );
        }
    }
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let ingress = world.session.admit_host_interaction_batch(pointer_batch(
        world.session.host_session.identity().as_u64(),
        presentation,
        5,
        [700_000, 500_000],
    ));
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = ingress
    else {
        panic!("empty viewport point must be admitted: {ingress:?}");
    };
    assert!(receipt.pointer_presence_denials().is_empty(), "{receipt:?}");
    let snapshot = world
        .session
        .interaction
        .pointer_presence_appearance_snapshot()
        .unwrap();
    assert_eq!(
        snapshot
            .postures()
            .iter()
            .find(|row| row.pointer() == UiHostPointerIdentity::new(5))
            .unwrap()
            .target(),
        None,
        "the viewport allocation must not replace occurrence hit bounds"
    );
    let _ = world.session.shutdown();
    assert_eq!(world.host.pending_presentation_count(), 0);
}

pub(super) fn pointer_batch(
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    [x, y]: [i64; 2],
) -> UiHostObservationBatch {
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol must negotiate");
    };
    let sequence = UiHostObservationSequence::new(sequence);
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            UiHostObservationPayload::PointerMotion {
                pointer: UiHostPointerIdentity::new(sequence.value()),
                capture_epoch: UiHostPointerCaptureEpoch::new(1),
                position: UiHostSurfacePosition::viewport_logical(x, y),
                pressed_buttons: UiHostPressedPointerButtons::NONE,
            },
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .unwrap()],
    })
    .unwrap()
}
