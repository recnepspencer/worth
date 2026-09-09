use super::*;
use crate::runtime::motion::{UiMotionCommitReceipt, UiMotionDeclaration, UiMotionTargetIdentity};

#[test]
fn focus_keyboard_and_placement_follow_current_motion_epoch_and_surface() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role);
    let (surface, graph) = super::super::mounting_fixture::mount(&mut session, 1_000);
    let neighbor = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            neighbor,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let node = session.mounted_graph_node(graph).unwrap();
    let unrelated = session.mount_instance(node, neighbor).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        neighbor,
    );
    fixture::close_source(&mut session, &role, "focus-motion-initial");
    session.advance_mounted_identity_frame().unwrap();
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("initial focus frame prepares"));
    fixture::publish(&mut session, &host, frame, 1);
    let original = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let other = session
        .mounted
        .current_presentation_for_surface(neighbor)
        .unwrap();
    let publication = session.mounted.current_publication().unwrap().clone();
    let instance = session
        .inspect_mounted_identity()
        .mounted_instances()
        .iter()
        .find(|instance| {
            instance.graph_node_identity() == graph
                && instance.basis().semantic_surface_identity() == surface
        })
        .unwrap()
        .identity();
    let target = UiMotionTargetIdentity::from_family_owner(surface, instance, 701);
    // Only the semantic Motion track is a fixture. Mounted completion, host
    // acceptance, keyboard ingress, Focus, placement, and appearance are real.
    // Fixed track geometry makes the sample physically presentable; this proof
    // claims epoch admission, not hit geometry or native pixel correctness.
    session
        .mounted
        .install_motion_commit(UiMotionCommitReceipt::for_sampling_test_transition(
            701,
            target,
            original,
            Some([0.0, 0.0, 100.0, 100.0]),
            true,
            Some([0.0, 0.0, 100.0, 100.0]),
            true,
            UiMotionDeclaration::portal_entrance(),
            None,
        ))
        .unwrap();
    host.push_presentation(UiHostSurfacePresentationOutcome::Presented(
        UiMountedSurfacePresentationCompletion::new(
            UiHostSurfacePresentationMode::NativeDisplay,
            UiHostPresentationEpoch::issued_by_host(2),
            UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
            UiHostPresentationCostReport::default(),
        ),
    ));
    let prepared = session.prepare_motion_tick(1, original).unwrap();
    session.present_prepared_motion_tick(prepared, original);
    let current = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    assert_ne!(current.epoch(), original.epoch());
    assert_eq!(
        publication.presentation_for_surface(surface),
        Some(original)
    );
    assert_eq!(
        session.mounted.current_presentation_for_surface(neighbor),
        Some(other)
    );

    applied(
        &mut session,
        current,
        1,
        UiHostObservationPayload::WindowFocus {
            surface: current.host_surface(),
            focused: true,
        },
    );
    let before = session.focus.as_ref().unwrap().appearance_posture();
    use crate::facade::observation_report::UiHostObservationReportDenial as Denial;
    for (sequence, invalid, expected) in [
        (2, original, Denial::PresentationEpochMismatch),
        (
            2,
            UiHostObservationPresentationBasis::new(
                other.host_surface(),
                current.frame(),
                current.binding(),
                current.epoch(),
            ),
            Denial::BindingNotPresented,
        ),
    ] {
        let outcome = ingress(&mut session, invalid, sequence, keyboard());
        let crate::facade::interaction::UiHostInteractionIngressOutcome::Denied(denial) = outcome
        else {
            panic!("invalid basis must be denied: {outcome:?}");
        };
        assert_eq!(denial.denial(), expected);
        assert_eq!(session.focus.as_ref().unwrap().appearance_posture(), before);
        assert!(host.last_focus_placement().is_none());
    }
    applied(&mut session, current, 2, keyboard());
    let first = session
        .focus
        .as_ref()
        .unwrap()
        .current_semantic_focus()
        .unwrap();
    assert_eq!(first.scope().semantic_surface(), surface);
    assert_ne!(first.mounted_instance(), unrelated);
    let placement = host
        .last_focus_placement()
        .expect("real host placement must occur");
    assert_eq!(placement.presentation(), current);
    assert_eq!(placement.target(), first.mounted_target());
    fixture::close_source(&mut session, &role, "focus-motion-current-close");
    drop(project(&mut session, &[(first.mounted_instance(), 30)]));

    // A valid event on a second surface must select that surface's scope,
    // even though the single semantic focus owner previously named the first.
    applied(&mut session, other, 3, keyboard());
    let second = session
        .focus
        .as_ref()
        .unwrap()
        .current_semantic_focus()
        .unwrap();
    assert_eq!(second.scope().semantic_surface(), neighbor);
    assert_eq!(second.mounted_instance(), unrelated);
    let placement = host.last_focus_placement().unwrap();
    assert_eq!(placement.presentation(), other);
    assert_eq!(placement.target(), second.mounted_target());
    fixture::close_source(&mut session, &role, "focus-motion-other-close");
    drop(project(
        &mut session,
        &[(first.mounted_instance(), 10), (unrelated, 30)],
    ));
    let _ = session.shutdown();
}

fn keyboard() -> UiHostObservationPayload {
    UiHostObservationPayload::Keyboard {
        logical_key: UiHostKey::Tab,
        physical_key: None,
        modifiers: UiHostKeyboardModifiers::default(),
        transition: UiHostKeyTransition::Pressed { repeat: false },
    }
}

fn applied(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    payload: UiHostObservationPayload,
) {
    let outcome = ingress(session, presentation, sequence, payload);
    assert!(
        matches!(
            outcome,
            crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(_)
        ),
        "current input sequence {sequence} must be applied: {outcome:?}"
    );
}

fn ingress(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    payload: UiHostObservationPayload,
) -> crate::facade::interaction::UiHostInteractionIngressOutcome {
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol");
    };
    let sequence = UiHostObservationSequence::new(sequence);
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: session.host_session.identity().as_u64(),
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            payload,
        )],
    })
    .unwrap();
    session.admit_host_interaction_batch(batch)
}
