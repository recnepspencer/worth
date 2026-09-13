use super::session::World;
use worth_ui_host_contract::*;

pub(super) fn exercise_window_focus(world: &mut World, first_sequence: u64) {
    let target = world
        .session
        .focus
        .as_ref()
        .unwrap()
        .current_semantic_focus()
        .expect("Portal0 establishes the exact Focus owner target")
        .mounted_instance();
    observe(world, first_sequence, false);
    assert_focus(
        world,
        crate::runtime::focus::UiFocusAppearanceClass::FocusedWindowInactive,
        target,
    );
    super::hostile_protocol::close_owner_snapshot(world);
}

pub(super) fn assert_inactive_publication(world: &World) {
    let target = world
        .session
        .focus
        .as_ref()
        .unwrap()
        .current_semantic_focus()
        .expect("inactive Focus retains the Portal0 owner target")
        .mounted_instance();
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .expect("accepted combined Portal frame retains inactive Focus appearance");
    assert!(output.fragments().iter().any(|fragment| matches!(
        fragment.identity(),
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            successor: Some(receipt),
            ..
        } if receipt.mounted_instance() == target
    )));
}

pub(super) fn assert_inactive_portal_selection(frame: &crate::mounting::UiPreparedMountedFrame) {
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        1,
        "Portal1 publication selects only the inactive Focus consumer"
    );
}

fn observe(world: &mut World, sequence: u64, focused: bool) {
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current host protocol")
    };
    let sequence = UiHostObservationSequence::new(sequence);
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: world.session.host_session.identity().as_u64(),
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            UiHostObservationPayload::WindowFocus {
                surface: presentation.host_surface(),
                focused,
            },
        )],
    })
    .unwrap();
    let outcome = world.session.admit_host_interaction_batch(batch);
    assert!(
        matches!(
            outcome,
            crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(_)
        ),
        "WindowFocus must reach the real Focus owner: {outcome:?}"
    );
}

fn assert_focus(
    world: &World,
    expected: crate::runtime::focus::UiFocusAppearanceClass,
    target: UiMountedInstanceIdentity,
) {
    let posture = world.session.focus.as_ref().unwrap().appearance_posture();
    assert_eq!(posture.class(), expected);
    assert_eq!(
        posture.target().unwrap().mounted_instance(),
        target,
        "window modality preserves the exact Portal-owned Focus target"
    );
}
