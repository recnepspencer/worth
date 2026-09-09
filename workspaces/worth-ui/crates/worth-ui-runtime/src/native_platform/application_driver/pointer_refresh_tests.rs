use super::{program_progress::UiNativePresentationSource, UiNativeApplicationDriver};
use crate::certification_support::{ScriptedPresentationHost, ScriptedSurfaceCompletion};
use crate::facade::WorthUiNativeApplicationShell;
use worth_ui_host_contract::*;
use worth_ui_host_native::{UiNativeEventLoopClient, UiNativeEventLoopDirective};
mod application_runtime;

pub(crate) fn exercise_custom_pointer_expiry(
    shell: WorthUiNativeApplicationShell,
    host: ScriptedPresentationHost,
    expires: u64,
    inspect: impl FnOnce(&WorthUiNativeApplicationShell),
) {
    use crate::native_platform::UiNativeApplicationRuntime;
    let mut runtime = Box::new(application_runtime::PointerApplicationRuntime);
    let shell = runtime
        .activate(shell, Box::new([]))
        .unwrap_or_else(|_| panic!("activate custom owner"));
    let mut driver = UiNativeApplicationDriver::from_launched_shell_for_test(shell);
    driver.application_runtime = Some(runtime);
    driver.application_runtime_active = true;
    driver
        .shell
        .as_mut()
        .unwrap()
        .install_native_observation_clock(
            worth_ui_host_native::UiNativeObservationClock::from_certification_elapsed(expires + 1)
                .unwrap(),
        )
        .unwrap();
    host.push_native_display_settled_without_effects();
    let outcome = driver.observation_time_ready().unwrap();
    assert_eq!(outcome.directive(), UiNativeEventLoopDirective::Close);
    assert_eq!(outcome.deadline(), None);
    assert_eq!(
        driver.progress.next_frame, 0,
        "custom owner does not enter the program lane"
    );
    assert!(!driver
        .shell
        .as_ref()
        .unwrap()
        .native_pointer_presentation_pending());
    inspect(driver.shell.as_ref().unwrap());
    assert!(driver.close_application_runtime().unwrap().is_some());
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PointerExpiryPresentation {
    Immediate,
    PendingRefresh,
    RejectedRefresh,
    PendingProgram,
}

// The calling confirmation world supplies a real issued challenge, admitted
// stationary target, and published Activation row. Only physical host I/O and
// elapsed clock epoch are simulated.
pub(crate) fn exercise_pointer_expiry(
    shell: WorthUiNativeApplicationShell,
    host: ScriptedPresentationHost,
    expires: u64,
    paint_token: &str,
    scenario: PointerExpiryPresentation,
) -> WorthUiNativeApplicationShell {
    let mut driver = UiNativeApplicationDriver::from_launched_shell_for_test(shell);
    let change = crate::runtime::tests::appearance_component_session_test_support::initial_appearance_theme_change(
        crate::capability::ThemeTokenId::new(paint_token).unwrap(),
        crate::capability::ThemeTokenValue::color(
            crate::capability::ThemeColorValue::hex("#315779").unwrap(),
        ),
    );
    let frame = if scenario == PointerExpiryPresentation::PendingProgram {
        crate::facade::entry::UiNativeApplicationFrame::with_theme_token_values([change.clone()])
            .unwrap()
    } else {
        crate::facade::entry::UiNativeApplicationFrame::present_current()
    };
    driver.progress = super::program_progress::UiNativeApplicationProgramProgress::new(
        crate::facade::entry::UiNativeApplicationProgram::new([frame])
            .unwrap()
            .remain_open_until_external_close(),
        None,
    );
    if scenario == PointerExpiryPresentation::PendingProgram {
        enqueue_pending(&host);
    } else {
        host.push_native_display_settled_without_effects();
    }
    driver
        .progress
        .advance(driver.shell.as_mut().unwrap())
        .unwrap();
    assert_eq!(driver.progress.next_frame, 1);
    if matches!(
        scenario,
        PointerExpiryPresentation::PendingRefresh | PointerExpiryPresentation::RejectedRefresh
    ) {
        // Static paint is still the Gate4 live publisher. A real color change
        // supplies physical work alongside the unpublished pointer successor.
        driver
            .shell
            .as_mut()
            .unwrap()
            .apply_theme_token_values(&[change])
            .unwrap();
        if scenario == PointerExpiryPresentation::PendingRefresh {
            enqueue_pending(&host);
        } else {
            host.push_presentation(UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                UiHostSurfacePresentationDenial::ExternalTimeout,
            ));
        }
    }
    driver
        .shell
        .as_mut()
        .unwrap()
        .install_native_observation_clock(
            worth_ui_host_native::UiNativeObservationClock::from_certification_elapsed(expires + 1)
                .unwrap(),
        )
        .unwrap();
    if scenario == PointerExpiryPresentation::Immediate {
        host.push_native_display_settled_without_effects();
    }
    let progress = driver
        .observation_time_ready()
        .expect("automatic timed close and producer handoff");
    assert_eq!(progress.deadline(), None);
    assert_eq!(progress.directive(), UiNativeEventLoopDirective::Continue);
    assert_eq!(
        driver.progress.next_frame, 1,
        "refresh does not consume authored frames"
    );
    let calls = host.presentation_calls();
    match scenario {
        PointerExpiryPresentation::Immediate => {}
        PointerExpiryPresentation::PendingRefresh | PointerExpiryPresentation::PendingProgram => {
            let expected_source = if scenario == PointerExpiryPresentation::PendingProgram {
                UiNativePresentationSource::Program(0)
            } else {
                UiNativePresentationSource::PointerRefresh
            };
            assert_eq!(driver.progress.pending.len(), 1);
            assert_eq!(
                driver.progress.pending.front().unwrap().source,
                expected_source
            );
            assert!(driver
                .shell
                .as_ref()
                .unwrap()
                .native_pointer_presentation_pending());
            driver.observation_time_ready().unwrap();
            assert_eq!(
                host.presentation_calls(),
                calls,
                "idle close must not replace in-flight work"
            );
            driver
                .progress
                .settle_first_pending_presentation_for_test(driver.shell.as_mut().unwrap())
                .unwrap();
            assert!(driver.progress.pending.is_empty());
            if scenario == PointerExpiryPresentation::PendingProgram {
                assert!(
                    driver
                        .shell
                        .as_ref()
                        .unwrap()
                        .native_pointer_presentation_pending(),
                    "older publication cannot acknowledge expiry"
                );
                host.push_native_display_settled_without_effects();
            }
            driver.observation_time_ready().unwrap();
        }
        PointerExpiryPresentation::RejectedRefresh => {
            assert_eq!(
                driver.progress.pending_retry.as_ref().unwrap().source,
                UiNativePresentationSource::PointerRefresh
            );
            assert!(driver
                .shell
                .as_ref()
                .unwrap()
                .native_pointer_presentation_pending());
            driver.observation_time_ready().unwrap();
            assert_eq!(host.presentation_calls(), calls);
            host.push_native_display_presented();
            driver.progress.observe_readiness(1, 1);
            driver
                .progress
                .advance(driver.shell.as_mut().unwrap())
                .unwrap();
            assert!(driver.progress.pending_retry.is_none());
            driver.observation_time_ready().unwrap();
        }
    }
    assert_eq!(driver.progress.next_frame, 1);
    let shell = driver.shell.as_ref().unwrap();
    assert!(!shell.native_pointer_presentation_pending());
    let calls = host.presentation_calls();
    driver.observation_time_ready().unwrap();
    assert_eq!(
        host.presentation_calls(),
        calls,
        "unchanged idle close has no physical work"
    );
    assert!(driver.progress.pending.is_empty());
    driver.shell.take().unwrap()
}

fn enqueue_pending(host: &ScriptedPresentationHost) {
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Presented(
            UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                crate::certification_support::scripted_presentation_epoch(),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                UiHostPresentationCostReport::from_adapter(UiHostPresentationCostInput {
                    presented_surfaces: 1,
                    asynchronous_handoffs: 1,
                    ..Default::default()
                }),
            ),
        )],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
}
