//! Rendering completion and wake delivery are scripted. Native input retention,
//! the driver's drain gate, mounted publication and runtime admission are real.
use crate::certification_support::{ScriptedPresentationHost, ScriptedSurfaceCompletion};
use crate::mounting::UiMountedFrameOutcome;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
};
use worth_ui_host_contract::*;
use worth_ui_host_native::{
    UiNativeEventLoopClient, UiNativeLifecycleEffect, UiNativeObservationReadinessGrant,
};

#[test]
fn delayed_frame_scroll_burst_does_not_disable_fresh_input_after_completion() {
    use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host_and_viewport_allocation;
    let host = ScriptedPresentationHost::native_display_with_native_input();
    let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host.clone())
        .launch_native_surface()
        .unwrap_or_else(|_| panic!("launch native shell"));
    shell.observe_native_viewport_readiness([800, 600], 1_000, false);
    super::super::program_progress::layout::complete_program_layout(&mut shell).unwrap();
    host.push_native_display_presented();
    assert!(matches!(
        shell
            .present_frame(u64::MAX, 1)
            .unwrap_or_else(|_| panic!("initial presentation")),
        UiMountedFrameOutcome::Published(_)
    ));
    let before = host
        .native_input_report()
        .last_completed_presentation()
        .unwrap();

    shell.observe_native_viewport_readiness([810, 600], 1_000, false);
    super::super::program_progress::layout::complete_program_layout(&mut shell).unwrap();
    host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(2),
                UiMountedCompletedEffects::new(Vec::new()),
                Default::default(),
            )),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let UiMountedFrameOutcome::InFlight(pending) = shell
        .present_frame(u64::MAX, 2)
        .unwrap_or_else(|_| panic!("pending successor presentation"))
    else {
        panic!("successor must be in flight")
    };
    assert!(!shell.native_observation_admission_ready());
    let mut driver = super::super::UiNativeApplicationDriver::from_launched_shell_for_test(shell);
    for generation in 1..=17 {
        host.observe_native_window_event(
            &WindowEvent::MouseWheel {
                device_id: DeviceId::dummy(),
                delta: MouseScrollDelta::LineDelta(0.0, -1.0),
                phase: TouchPhase::Moved,
            },
            generation,
            Some(PhysicalPosition::new(100.0, 100.0)),
        );
        driver
            .native_observations_ready(UiNativeObservationReadinessGrant::from_certification(
                generation,
            ))
            .unwrap();
    }
    assert_eq!(
        driver.observation_ingress_counts, [0; 5],
        "the real driver gate blocks drain"
    );
    assert!(
        host.begin_native_input_recovery().is_none(),
        "cannot cancel ahead of retained input"
    );
    assert_eq!(
        host.native_input_report().last_completed_presentation(),
        Some(before)
    );
    let shell = driver.shell.as_mut().unwrap();
    let UiMountedFrameOutcome::InFlight(pending) = shell.complete_frame_presentation(pending, 20)
    else {
        panic!("first poll must remain pending")
    };
    assert!(!shell.native_observation_admission_ready());
    match shell.complete_frame_presentation(pending, 21) {
        UiMountedFrameOutcome::Published(_)
        | UiMountedFrameOutcome::Unchanged(_)
        | UiMountedFrameOutcome::Reconciled(_) => {}
        UiMountedFrameOutcome::CompletionDenied(denial) => panic!("completion denied: {denial:?}"),
        UiMountedFrameOutcome::Superseded(_) => panic!("successor superseded"),
        UiMountedFrameOutcome::PresentationIndeterminate(_) => panic!("completion indeterminate"),
        _ => panic!("successor did not complete"),
    }
    assert!(shell.native_observation_admission_ready());
    assert_ne!(
        host.native_input_report().last_completed_presentation(),
        Some(before)
    );
    driver
        .native_observations_ready(UiNativeObservationReadinessGrant::from_certification(18))
        .unwrap();
    assert_eq!(
        driver.observation_ingress_counts,
        [16, 0, 0, 0, 0],
        "driver drains retained input after completion"
    );
    let grant = host
        .begin_native_input_recovery()
        .expect("drained overflow admits recovery");
    let (acknowledgement, _) = driver.native_input_retention_exhausted(grant).unwrap();
    assert!(host.complete_native_input_recovery(acknowledgement));
    assert_eq!(host.native_input_report().terminal_stop(), None);
    let fresh = host.observe_native_window_event(
        &WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        },
        30,
        Some(PhysicalPosition::new(100.0, 100.0)),
    );
    let stop = host.native_input_report().terminal_stop();
    driver
        .native_observations_ready(UiNativeObservationReadinessGrant::from_certification(19))
        .unwrap();
    assert_eq!(
        driver.observation_ingress_counts,
        [17, 0, 0, 0, 0],
        "fresh input reaches runtime admission"
    );
    let shutdown = driver.shell.take().unwrap().shutdown();
    assert!(shutdown.host_session_released());
    assert_eq!(
        fresh.effect(),
        UiNativeLifecycleEffect::Retained,
        "input must recover after backpressure clears; stop={stop:?}"
    );
}
