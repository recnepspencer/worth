use crate::certification_support::ScriptedPresentationHost;
use crate::mounting::{UiMountedFrameOutcome, UiMountedFramePublicationReceipt};
use crate::runtime::tests::active_application_session_test_support::{
    source_backed_component_session, source_backed_focusable_component_app_with_host,
};
use crate::runtime::tests::native_pointer_observation_test_support::source_backed_hover_consumer_app_with_host;
use worth_ui_host_contract::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationLoss,
    UiHostObservationPayload, UiHostObservationPresentationBasis, UiHostObservationReport,
    UiHostObservationSequence, UiHostObservationSequenceRange, UiHostObservationTimeBasis,
    UiHostPointerCaptureEpoch, UiHostPointerDeviceKind, UiHostPointerIdentity,
    UiHostPresentationEpoch, UiHostPressedPointerButtons, UiHostProtocolContract,
    UiHostProtocolNegotiation, UiHostSurfacePosition, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

#[test]
fn native_observation_ready_path_drains_through_runtime_interaction_owner() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_focusable_component_app_with_host(host.clone())
        .launch_native_surface()
        .expect("native shell should launch");

    let first = published(shell.present_frame(100, 1), "first");
    assert_eq!(
        shell
            .session
            .focus
            .as_ref()
            .expect("focusable fixture installs Focus")
            .participant_count_for_test(),
        1
    );
    let binding = *first.bindings().first().expect("native binding");
    let host_surface = shell.session.mounted.view().surface_bindings()[0].host_surface_identity();
    let batch = focus_batch(
        shell.session.host_session.identity().as_u64(),
        UiHostObservationPresentationBasis::new(
            host_surface,
            first.frame(),
            binding,
            UiHostPresentationEpoch::issued_by_host(1),
        ),
    );
    host.enqueue_observation_for_next_drain(batch);
    let settlement = shell.admit_native_observation_batches(Default::default());
    assert_eq!(settlement.counts(), (1, 0, 0, 0));
    assert_eq!(settlement.drain_denial(), None);
    let outcomes = settlement.into_outcomes();
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        &outcomes[0],
        crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(_)
    ));
    assert!(shell
        .session
        .focus
        .as_ref()
        .expect("focusable fixture installs Focus")
        .window_is_focused_for_test());

    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
}

#[test]
fn pointer_motion_publishes_only_owner_issued_target_changes() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_hover_consumer_app_with_host(host.clone())
        .launch_native_surface()
        .expect("pointer-presence shell should launch");
    let frame = published(shell.present_frame(100, 1), "hover");
    let binding = *frame.bindings().first().expect("native binding");
    let host_surface = shell.session.mounted.view().surface_bindings()[0].host_surface_identity();
    let presentation = UiHostObservationPresentationBasis::new(
        host_surface,
        frame.frame(),
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let hit_test = shell
        .session
        .mounted
        .interaction_hit_test_basis(presentation)
        .expect("the published frame should expose its mounted hit-test basis");
    let row = hit_test
        .rows()
        .first()
        .expect("the mounted component should expose a hit-test row");
    let bounds = row.bounds();
    let clip = row.clip_bounds();
    let point = UiHostSurfacePosition::viewport_logical(
        (((bounds.x().max(clip.x()) + (bounds.x() + bounds.width()).min(clip.x() + clip.width()))
            / 2.0)
            * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32) as i64,
        (((bounds.y().max(clip.y())
            + (bounds.y() + bounds.height()).min(clip.y() + clip.height()))
            / 2.0)
            * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32) as i64,
    );

    let first = match shell
        .session
        .admit_host_interaction_batch(pointer_motion_batch(
            shell.session.host_session.identity().as_u64(),
            presentation,
            1,
            point,
        )) {
        crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) => receipt,
        _ => panic!("current pointer motion should reach the interaction owner"),
    };
    assert_eq!(first.pointer_presence_transitions().len(), 1);
    let transition = first.pointer_presence_transitions()[0].clone();
    let mut turn = shell.session.begin_observation_turn().unwrap();
    turn.admit_pointer_presence_transition(transition.clone())
        .unwrap();
    let observations = turn.seal().unwrap();
    let changed = match shell.session.classify_observations(observations).unwrap() {
        crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("an owner-issued target change must publish one semantic fact"),
    };
    assert_eq!(changed.facts().len(), 1);
    assert!(matches!(
        &changed.facts()[0],
        crate::fact_contract::UiProducedFact::PointerPresenceTarget(fact)
            if fact.owner_revision() == 1 && fact.current().is_some()
    ));

    let repeated = match shell
        .session
        .admit_host_interaction_batch(pointer_motion_batch(
            shell.session.host_session.identity().as_u64(),
            presentation,
            2,
            UiHostSurfacePosition::viewport_logical(point.x_subpixels() + 1, point.y_subpixels()),
        )) {
        crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) => receipt,
        _ => panic!("same-target pointer motion should remain valid physical evidence"),
    };
    assert!(repeated.pointer_presence_transitions().is_empty());

    let departure = match shell
        .session
        .admit_host_interaction_batch(pointer_motion_batch(
            shell.session.host_session.identity().as_u64(),
            presentation,
            3,
            UiHostSurfacePosition::viewport_logical(
                ((bounds.x() + bounds.width() + 1.0)
                    * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32) as i64,
                point.y_subpixels(),
            ),
        )) {
        crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) => receipt,
        _ => panic!("pointer departure should reach the installed interaction owner"),
    };
    assert_eq!(departure.pointer_presence_transitions().len(), 1);
    let reentry = match shell
        .session
        .admit_host_interaction_batch(pointer_motion_batch(
            shell.session.host_session.identity().as_u64(),
            presentation,
            4,
            point,
        )) {
        crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) => receipt,
        _ => panic!("pointer reentry should reach the installed interaction owner"),
    };
    assert_eq!(reentry.pointer_presence_transitions().len(), 1);
    let stale_transition = reentry.pointer_presence_transitions()[0].clone();
    let mut stale_turn = shell.session.begin_observation_turn().unwrap();
    stale_turn
        .admit_pointer_presence_transition(stale_transition.clone())
        .unwrap();
    let stale_set = stale_turn.seal().unwrap();
    let stale_instance = stale_transition.current().expect("owner transition target");
    let stale_basis = shell
        .session
        .mounted
        .current_mounted_identity_basis(stale_instance)
        .expect("owner target should still be mounted")
        .clone();
    let stale_node = shell
        .session
        .mounted_graph_node(stale_basis.graph_node_identity())
        .expect("owner target graph node should remain current");
    shell.session.unmount_instance(stale_instance).unwrap();
    shell
        .session
        .mount_instance(stale_node, stale_basis.semantic_surface_identity())
        .expect("the same graph node should remount with a successor receipt");
    assert!(matches!(
        shell.session.classify_observations(stale_set),
        Err(
            crate::runtime::observation::UiChangeClassificationDenial::StalePointerPresenceTransition
        )
    ));

    let mut foreign = source_backed_component_session();
    let mut foreign_turn = foreign.begin_observation_turn().unwrap();
    foreign_turn
        .admit_pointer_presence_transition(transition)
        .unwrap();
    let foreign_set = foreign_turn.seal().unwrap();
    assert!(matches!(
        foreign.classify_observations(foreign_set),
        Err(
            crate::runtime::observation::UiChangeClassificationDenial::ForeignApplicationGeneration
        )
    ));
    let _ = foreign.shutdown();
    let _ = shell.shutdown();
}

fn published(
    outcome: Result<
        UiMountedFrameOutcome,
        crate::facade::entry::WorthUiMountedFrameExecutionStop<'_>,
    >,
    label: &str,
) -> UiMountedFramePublicationReceipt {
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(_) => panic!("{label} frame should execute"),
    };
    match outcome {
        UiMountedFrameOutcome::Published(receipt)
        | UiMountedFrameOutcome::Unchanged(receipt)
        | UiMountedFrameOutcome::Reconciled(receipt) => receipt,
        UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("{label} frame was rejected before effects")
        }
        UiMountedFrameOutcome::InFlight(_) => panic!("{label} frame remained in flight"),
        UiMountedFrameOutcome::Superseded(_) => panic!("{label} frame was superseded"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("{label} frame became indeterminate")
        }
        UiMountedFrameOutcome::RetentionDenied(_) => panic!("{label} frame retention was denied"),
        UiMountedFrameOutcome::AdmissionDenied(_) => panic!("{label} frame admission was denied"),
        UiMountedFrameOutcome::CompletionDenied(_) => panic!("{label} frame completion was denied"),
    }
}

fn focus_batch(
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
) -> UiHostObservationBatch {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let sequence = UiHostObservationSequence::new(1);
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(2),
            UiHostObservationPayload::WindowFocus {
                surface: presentation.host_surface(),
                focused: true,
            },
        )],
    })
    .expect("focus observation batch should satisfy the host contract")
}

fn pointer_motion_batch(
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    position: UiHostSurfacePosition,
) -> UiHostObservationBatch {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
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
                pointer: UiHostPointerIdentity::new(1),
                capture_epoch: UiHostPointerCaptureEpoch::new(1),
                pressed_buttons: UiHostPressedPointerButtons::NONE,
                position,
            },
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .expect("the positive native pointer fixture declares its kind")],
    })
    .expect("pointer observation batch should satisfy the host contract")
}
