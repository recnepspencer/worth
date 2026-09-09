#[path = "tests/shutdown.rs"]
mod shutdown;

#[cfg(feature = "certification-support")]
#[test]
fn external_timeout_retains_the_rejected_frame_until_a_later_redraw_succeeds() {
    let (host, mut shell, mut progress) = retryable_program();
    host.push_presentation(rejected(
        worth_ui_host_contract::UiHostSurfacePresentationDenial::ExternalTimeout,
    ));
    host.push_native_display_presented();

    progress.observe_readiness(1, 1);
    progress
        .advance(&mut shell)
        .expect("the rejected frame must become retained retry work");
    let retained_frame = progress
        .pending_retry
        .as_ref()
        .expect("timeout must retain the rejected frame")
        .rejected
        .frame()
        .canonical_core();
    assert_eq!(
        progress.retry_readiness(),
        Some(super::program_progress::UiNativeProgramRetryReadiness::Timeout)
    );
    assert_eq!(progress.next_frame, 0);
    assert_eq!(host.presentation_calls(), 1);

    progress
        .advance(&mut shell)
        .expect("the same readiness generation must remain parked");
    assert_eq!(host.presentation_calls(), 1);
    assert_eq!(
        progress
            .pending_retry
            .as_ref()
            .expect("parked retry must retain its frame")
            .rejected
            .frame()
            .canonical_core(),
        retained_frame
    );

    progress.observe_readiness(2, 2);
    progress
        .advance(&mut shell)
        .expect("a later redraw must retry the retained frame");
    assert!(progress.pending_retry.is_none());
    assert_eq!(progress.next_frame, 1);
    assert_eq!(host.presentation_calls(), 2);
}

#[cfg(feature = "certification-support")]
#[test]
fn retry_that_becomes_in_flight_advances_the_program_frame_exactly_once() {
    use crate::certification_support::ScriptedSurfaceCompletion;
    use crate::facade::mounted::UiHostSurfaceCancellationOutcome;

    let (host, mut shell, mut progress) = retryable_program();
    host.push_presentation(rejected(
        worth_ui_host_contract::UiHostSurfacePresentationDenial::ExternalTimeout,
    ));
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );

    progress.observe_readiness(1, 1);
    progress
        .advance(&mut shell)
        .expect("timeout must park retry");
    progress.observe_readiness(2, 2);
    progress
        .advance(&mut shell)
        .expect("later readiness must submit the retained frame once");
    assert!(progress.pending_retry.is_none());
    assert_eq!(progress.pending.len(), 1);
    assert_eq!(progress.next_frame, 1);
    assert_eq!(host.presentation_calls(), 2);

    progress
        .advance(&mut shell)
        .expect("in-flight retry must not replay its logical frame");
    assert_eq!(progress.pending.len(), 1);
    assert_eq!(progress.next_frame, 1);
    assert_eq!(host.presentation_calls(), 2);
}

#[cfg(feature = "certification-support")]
#[test]
fn owner_reconstruction_settles_the_current_program_frame_without_a_parallel_retry_lane() {
    use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host;

    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let mut shell = source_backed_component_app_with_host(host.clone())
        .launch_native_surface()
        .expect("native certification shell should launch");
    let program = crate::facade::entry::UiNativeApplicationProgram::new([
        crate::facade::entry::UiNativeApplicationFrame::present_current(),
        crate::facade::entry::UiNativeApplicationFrame::present_current(),
    ])
    .expect("two settled frames are a valid application program");
    let mut progress =
        super::program_progress::UiNativeApplicationProgramProgress::new(program, None);
    host.push_native_display_presented();
    host.push_presentation(rejected(
        worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired,
    ));
    host.push_native_display_presented();

    progress.observe_readiness(1, 1);
    progress
        .advance(&mut shell)
        .expect("owner reconstruction must settle through current mounted authority");
    assert!(progress.pending_retry.is_none());
    assert_eq!(progress.next_frame, 2);
    assert_eq!(host.presentation_calls(), 3);
}

#[cfg(feature = "certification-support")]
#[test]
fn nested_reconstruction_retains_physical_authority_until_in_flight_settlement() {
    use crate::certification_support::ScriptedSurfaceCompletion;
    use crate::facade::mounted::{
        UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationMode, UiMountedCompletedEffects,
        UiMountedEffectFamily,
    };

    let (host, mut shell, mut progress) = retryable_program();
    host.push_native_display_presented();
    progress.observe_readiness(1, 1);
    progress
        .advance(&mut shell)
        .expect("the current mounted authority must first publish");
    host.push_presentation(rejected(
        worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired,
    ));
    assert!(matches!(
        shell.present_frame(u64::MAX, 2),
        Ok(crate::facade::mounted::UiMountedFrameOutcome::RejectedBeforeEffects(_))
    ));

    let recovery_attempt =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let recovery =
        worth_ui_host_native::UiNativePhysicalPresentationCorrelation::from_certification(
            recovery_attempt,
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            1,
        )
        .unwrap();
    progress
        .physical_recovery
        .expect(recovery.attempt(), recovery.binding())
        .unwrap();
    progress
        .physical_recovery
        .observe_scheduled(recovery)
        .unwrap();

    host.push_presentation(rejected(
        worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired,
    ));
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Presented(
            worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                crate::certification_support::scripted_presentation_epoch(),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                worth_ui_host_contract::UiHostPresentationCostReport::from_adapter(
                    worth_ui_host_contract::UiHostPresentationCostInput {
                        presented_surfaces: 1,
                        asynchronous_handoffs: 1,
                        ..Default::default()
                    },
                ),
            ),
        )],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    progress
        .resume_reconstruction(
            &mut shell,
            super::program_progress::UiNativePresentationSource::Program(0),
            recovery,
        )
        .expect("nested reconstruction must retain the in-flight successor");

    let pending = progress.pending.front().expect("in-flight successor");
    assert_eq!(
        pending.reconstruction_authority,
        Some(super::program_progress::UiNativeProgramReconstructionAuthority::Physical(recovery))
    );
    progress
        .settle_first_pending_presentation_for_test(&mut shell)
        .expect("physical authority settles exactly once");
    assert!(progress.physical_recovery.is_empty());
    assert_eq!(progress.next_frame, 1);
    assert!(progress.should_close());
    assert_eq!(host.presentation_calls(), 4);
}

#[cfg(feature = "certification-support")]
#[test]
fn occlusion_waits_for_a_later_visibility_readiness_before_retrying() {
    let (host, mut shell, mut progress) = retryable_program();
    host.push_presentation(rejected(
        worth_ui_host_contract::UiHostSurfacePresentationDenial::SurfaceOccluded,
    ));
    host.push_native_display_presented();

    progress.observe_readiness(7, 7);
    progress
        .advance(&mut shell)
        .expect("occlusion must park the rejected frame");
    assert_eq!(
        progress.retry_readiness(),
        Some(super::program_progress::UiNativeProgramRetryReadiness::Visibility)
    );
    progress
        .advance(&mut shell)
        .expect("visibility must not be invented within one readiness generation");
    assert_eq!(host.presentation_calls(), 1);

    progress.observe_readiness(8, 8);
    progress
        .advance(&mut shell)
        .expect("later visibility readiness must retry the retained frame");
    assert!(progress.pending_retry.is_none());
    assert_eq!(progress.next_frame, 1);
    assert_eq!(host.presentation_calls(), 2);
}

#[cfg(feature = "certification-support")]
#[test]
fn dpi_timeout_keeps_the_successor_binding_until_retry_settles_reconciliation() {
    let (host, mut shell, mut progress) = retryable_program();
    shell.observe_native_viewport_readiness([800, 600], 1_000, false);
    shell
        .rebind_native_surface_scale(2_000)
        .expect("DPI successor must establish a replacement binding");
    host.push_presentation(rejected(
        worth_ui_host_contract::UiHostSurfacePresentationDenial::ExternalTimeout,
    ));
    host.push_native_display_presented();

    progress.observe_readiness(11, 11);
    progress
        .advance(&mut shell)
        .expect("the replacement-binding frame must become retry work");
    let replacement_binding = progress
        .pending_retry
        .as_ref()
        .expect("timeout must retain the reconciliation frame")
        .rejected
        .rejections()[0]
        .binding();
    assert_ne!(replacement_binding.diagnostic_value(), 0);
    assert!(shell.rebind_native_surface_scale(3_000).is_err());

    progress.observe_readiness(12, 12);
    progress
        .advance(&mut shell)
        .expect("later readiness must settle the retained replacement binding");
    assert!(progress.pending_retry.is_none());
    assert_eq!(progress.next_frame, 1);
    assert_eq!(host.presentation_calls(), 2);
    assert!(shell.rebind_native_surface_scale(3_000).is_ok());
}

#[cfg(feature = "certification-support")]
#[test]
fn surface_successor_frames_ignore_same_basis_redraw_generations() {
    use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host;

    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    for _ in 0..8 {
        host.push_native_display_presented();
    }
    let mut shell = source_backed_component_app_with_host(host.clone())
        .launch_native_surface()
        .expect("native test shell should launch");
    let program = crate::facade::entry::UiNativeApplicationProgram::new([
        crate::facade::entry::UiNativeApplicationFrame::present_current(),
        crate::facade::entry::UiNativeApplicationFrame::present_current()
            .after_host_surface_basis_successor(),
        crate::facade::entry::UiNativeApplicationFrame::present_current()
            .after_host_surface_basis_successor(),
    ])
    .expect("three bounded surface-basis frames");
    let mut progress = super::program_progress::UiNativeApplicationProgramProgress::new(
        program.remain_open_until_external_close(),
        None,
    );

    progress.observe_readiness(1, 1);
    progress.advance(&mut shell).unwrap();
    assert_eq!(progress.next_frame, 1);
    progress.observe_readiness(2, 2);
    progress.advance(&mut shell).unwrap();
    assert_eq!(progress.next_frame, 2);
    let calls_before_same_basis = host.presentation_calls();
    progress.observe_readiness(3, 2);
    progress.advance(&mut shell).unwrap();
    assert_eq!(progress.next_frame, 2);
    assert_eq!(host.presentation_calls(), calls_before_same_basis);
    progress.observe_readiness(4, 3);
    progress.advance(&mut shell).unwrap();
    assert_eq!(progress.next_frame, 3);
    assert!(host.presentation_calls() > calls_before_same_basis);
}

#[test]
fn installed_but_inactive_motion_does_not_arm_native_readiness() {
    use std::{sync::mpsc, time::Duration};

    use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host;

    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let shell = source_backed_component_app_with_host(host)
        .launch_native_surface()
        .expect("native test shell should launch without active Motion tracks");
    let mut driver = super::UiNativeApplicationDriver::from_launched_shell_for_test(shell);
    driver.motion_support_installed = true;

    let (signal_sender, signals) = mpsc::channel();
    driver.motion_readiness = Some(super::UiNativeMotionReadinessLane::start_for_test(
        move || signal_sender.send(()).map_err(|_| ()),
    ));

    assert_eq!(driver.application_readiness_owner_count().get(), 1);
    assert!(!driver.motion_can_request_readiness());
    driver.arm_motion_readiness_now();
    assert!(matches!(
        signals.recv_timeout(Duration::from_millis(60)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));

    driver
        .motion_readiness
        .take()
        .expect("installed Motion owns its readiness lane")
        .shutdown();
}

#[cfg(feature = "certification-support")]
fn retryable_program() -> (
    crate::certification_support::ScriptedPresentationHost,
    crate::facade::WorthUiNativeApplicationShell,
    super::program_progress::UiNativeApplicationProgramProgress,
) {
    use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host;

    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let shell = source_backed_component_app_with_host(host.clone())
        .launch_native_surface()
        .expect("native certification shell should launch");
    let progress = super::program_progress::UiNativeApplicationProgramProgress::new(
        crate::facade::entry::UiNativeApplicationProgram::single_frame(),
        None,
    );
    (host, shell, progress)
}

#[cfg(feature = "certification-support")]
fn rejected(
    denial: worth_ui_host_contract::UiHostSurfacePresentationDenial,
) -> crate::facade::mounted::UiHostSurfacePresentationOutcome {
    crate::facade::mounted::UiHostSurfacePresentationOutcome::RejectedBeforeEffects(denial)
}
