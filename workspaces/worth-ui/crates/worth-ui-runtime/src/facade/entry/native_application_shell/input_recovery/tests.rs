//! Native input and runtime gestures are real; layout and rendering use the
//! existing pointer-admission fixture, not an OS window or recorder.
use crate::certification_support::ScriptedPresentationHost;
use crate::facade::entry::WorthUiNativeApplicationShell;
use crate::runtime::tests::native_pointer_observation_test_support::source_backed_hover_consumer_app_with_host;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
};
use worth_ui_host_native::UiNativeLifecycleEffect;

#[test]
fn dropped_release_cancels_gesture_before_fresh_input_can_activate() {
    let host = ScriptedPresentationHost::native_display_with_native_input();
    let mut shell = source_backed_hover_consumer_app_with_host(host.clone())
        .launch_native_surface()
        .unwrap();
    crate::facade::entry::native_application_identity_trace_test_support::install_bound_surface_geometry(&mut shell);
    host.push_native_display_presented();
    assert!(matches!(
        shell.present_frame(100, 1),
        Ok(crate::mounting::UiMountedFrameOutcome::Published(_))
    ));
    let basis = host
        .native_input_report()
        .last_completed_presentation()
        .unwrap();
    let hit_test = shell
        .session
        .mounted
        .interaction_hit_test_basis(basis)
        .unwrap();
    let row = hit_test.rows().first().unwrap();
    let bounds = row.bounds().platform_box();
    let clip = row.clip_bounds().platform_box();
    let point = PhysicalPosition::new(
        f64::from(
            (bounds.x().max(clip.x()) + (bounds.x() + bounds.width()).min(clip.x() + clip.width()))
                / 2.0,
        ),
        f64::from(
            (bounds.y().max(clip.y())
                + (bounds.y() + bounds.height()).min(clip.y() + clip.height()))
                / 2.0,
        ),
    );
    assert_eq!(
        button(&host, ElementState::Pressed, point, 2),
        UiNativeLifecycleEffect::Retained
    );
    for tick in 3..18 {
        assert_eq!(
            host.observe_native_window_event(
                &WindowEvent::MouseWheel {
                    device_id: DeviceId::dummy(),
                    delta: MouseScrollDelta::LineDelta(0.0, -1.0),
                    phase: TouchPhase::Moved,
                },
                tick,
                Some(point)
            )
            .effect(),
            UiNativeLifecycleEffect::Retained
        );
    }
    // The release is the first lost event. A drain alone leaves the admitted
    // press active, so re-enabling delivery alone is not a safe recovery.
    assert_ne!(
        button(&host, ElementState::Released, point, 18),
        UiNativeLifecycleEffect::Retained
    );
    assert!(host.begin_native_input_recovery().is_none());
    assert_eq!(
        shell
            .admit_native_observation_batches(Default::default())
            .counts(),
        (16, 0, 0, 0)
    );
    assert_eq!(shell.session.interaction_state().active_gestures(), 1);
    let grant = host.begin_native_input_recovery().unwrap();
    let settlement = shell.cancel_exhausted_native_input(&grant).unwrap();
    assert_eq!(settlement.counts(), (0, 0, 0, 1));
    assert_eq!(shell.session.interaction_state().active_gestures(), 0);
    assert!(host.complete_native_input_recovery(grant.acknowledge_cancellation()));
    assert_eq!(
        button(&host, ElementState::Released, point, 19),
        UiNativeLifecycleEffect::Retained
    );
    assert_eq!(
        shell
            .admit_native_observation_batches(Default::default())
            .counts(),
        (1, 0, 0, 0)
    );
    assert_eq!(
        activations(&shell),
        0,
        "late release cannot complete the cancelled press"
    );
    for (tick, state) in [(20, ElementState::Pressed), (21, ElementState::Released)] {
        assert_eq!(
            button(&host, state, point, tick),
            UiNativeLifecycleEffect::Retained
        );
        assert_eq!(
            shell
                .admit_native_observation_batches(Default::default())
                .counts(),
            (1, 0, 0, 0)
        );
    }
    assert_eq!(
        activations(&shell),
        1,
        "a new complete gesture remains usable"
    );
    assert!(shell.shutdown().host_session_released());
}

fn button(
    host: &ScriptedPresentationHost,
    state: ElementState,
    point: PhysicalPosition<f64>,
    tick: u64,
) -> UiNativeLifecycleEffect {
    host.observe_native_window_event(
        &WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state,
            button: MouseButton::Left,
        },
        tick,
        Some(point),
    )
    .effect()
}

fn activations(shell: &WorthUiNativeApplicationShell) -> u64 {
    shell
        .session
        .interaction_state()
        .counters()
        .semantic_interactions()
}
