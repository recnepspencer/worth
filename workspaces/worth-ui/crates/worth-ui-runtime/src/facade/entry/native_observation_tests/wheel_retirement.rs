use super::*;
use crate::facade::interaction::UiHostInteractionIngressOutcome;
use winit::event::{MouseScrollDelta, TouchPhase};

#[test]
fn delivered_native_wheel_bursts_release_capacity_and_preserve_later_buttons() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_hover_consumer_app_with_host(host.clone())
        .launch_native_surface()
        .unwrap();
    super::super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );
    let frame = published(shell.present_frame(100, 1), "wheel");
    let presentation = UiHostObservationPresentationBasis::new(
        shell.session.mounted.view().surface_bindings()[0].host_surface_identity(),
        frame.frame(),
        frame.bindings()[0],
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let host_session = shell.session.host_session.identity().as_u64();
    let mut input = worth_ui_host_native::UiNativeInputObservationContract::new();
    input.install_initial_profile(1.0, [800, 600]);
    assert!(input.record_completed_presentation(
        shell.session.host_session.protocol(),
        host_session,
        presentation,
    ));
    let position = PhysicalPosition::new(10.0, 20.0);
    for burst in 0..8 {
        for step in 0..16 {
            assert_eq!(
                input.observe_window_event_at(
                    &WindowEvent::MouseWheel {
                        device_id: DeviceId::dummy(),
                        delta: MouseScrollDelta::LineDelta(
                            0.0,
                            if burst % 2 == 0 { -1.0 } else { 1.0 }
                        ),
                        phase: TouchPhase::Moved,
                    },
                    2 + burst * 16 + step,
                    Some(position),
                ),
                worth_ui_host_native::UiNativeInputObservationContractDisposition::Retained
            );
        }
        for batch in input.drain(host_session).into_batches().into_vec() {
            host.enqueue_observation_for_next_drain(batch);
        }
        let settlement = shell.admit_native_observation_batches(Default::default());
        assert_eq!(settlement.counts(), (16, 0, 0, 0), "burst {burst}");
        assert_eq!(shell.session.retained_host_observation_report_count(), 0);
        assert_eq!(shell.session.retained_host_observation_byte_count(), 0);
    }
    for (step, state) in [ElementState::Pressed, ElementState::Released]
        .into_iter()
        .enumerate()
    {
        input.observe_window_event_at(
            &WindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state,
                button: MouseButton::Left,
            },
            200 + step as u64,
            Some(position),
        );
    }
    let batches = input.drain(host_session).into_batches().into_vec();
    let duplicate = batches[1].clone();
    for batch in batches {
        host.enqueue_observation_for_next_drain(batch);
    }
    assert_eq!(
        shell
            .admit_native_observation_batches(Default::default())
            .counts(),
        (2, 0, 0, 0)
    );
    assert_eq!(shell.session.interaction_state().active_gestures(), 0);
    assert!(matches!(
        shell.session.admit_host_interaction_batch(duplicate),
        UiHostInteractionIngressOutcome::Duplicate(_)
    ));
    assert_eq!(
        shell
            .session
            .interaction_state()
            .counters()
            .button_reports(),
        2
    );
    let sequence = UiHostObservationSequence::new(132);
    let gap = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol: shell.session.host_session.protocol(),
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(300),
            UiHostObservationPayload::WindowFocus {
                surface: presentation.host_surface(),
                focused: true,
            },
        )],
    })
    .unwrap();
    assert!(matches!(shell.session.admit_host_interaction_batch(gap),
        UiHostInteractionIngressOutcome::Denied(denial)
            if denial.denial() == crate::facade::observation_report::UiHostObservationReportDenial::SequenceGap));
    assert!(shell.shutdown().host_session_released());
}
