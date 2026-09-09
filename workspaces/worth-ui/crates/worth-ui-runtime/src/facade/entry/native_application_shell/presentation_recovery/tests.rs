use crate::facade::mounted::{
    UiHostSurfacePresentationDenial, UiHostSurfacePresentationOutcome, UiMountedFrameOutcome,
};
#[cfg(feature = "certification-support")]
use crate::inspection::mounted_frame::{UiMountedInspectionReceipt, UiMountedInspectionRequest};

#[test]
fn host_required_reconstruction_recovers_through_current_mounted_authority() {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let mut shell =
        crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host(
            host.clone(),
        )
        .launch_native_surface()
        .expect("native certification shell launches");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);

    host.push_native_display_presented();
    assert!(matches!(
        shell
            .present_frame(2, 1)
            .unwrap_or_else(|_| panic!("initial frame executes")),
        UiMountedFrameOutcome::Published(_)
    ));

    host.push_presentation(UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
        UiHostSurfacePresentationDenial::ReconstructionRequired,
    ));
    let rejected = shell
        .present_frame(4, 3)
        .unwrap_or_else(|_| panic!("successor frame executes"));
    assert!(matches!(
        rejected,
        UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));

    host.push_native_display_presented();
    let recovered = shell
        .recover_reconstruction_required_presentation(rejected, 6, 5)
        .expect("host-required reconstruction remains available");
    assert!(matches!(
        recovered,
        UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Reconciled(_)
    ));
    assert_eq!(host.presentation_calls(), 3);
}

#[test]
fn non_reconstruction_rejection_is_returned_without_an_extra_host_attempt() {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let mut shell =
        crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host(
            host.clone(),
        )
        .launch_native_surface()
        .expect("native certification shell launches");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);
    host.push_presentation(UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
        UiHostSurfacePresentationDenial::ExternalTimeout,
    ));
    let rejected = shell
        .present_frame(2, 1)
        .unwrap_or_else(|_| panic!("frame executes"));
    let returned = shell
        .recover_reconstruction_required_presentation(rejected, 4, 3)
        .expect("unrelated rejection remains an ordinary outcome");
    assert!(matches!(
        returned,
        UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    assert_eq!(host.presentation_calls(), 1);
}

#[cfg(feature = "certification-support")]
#[test]
fn indeterminate_recovery_waits_for_the_exact_physical_correlation() {
    use crate::certification_support::ScriptedSurfaceCompletion;
    use crate::facade::entry::WorthUiNativePhysicalPresentationRecovery;
    use crate::facade::mounted::UiHostSurfaceCancellationOutcome;

    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let mut shell =
        crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host(
            host.clone(),
        )
        .launch_native_surface()
        .expect("native certification shell launches");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);

    host.push_native_display_presented();
    assert!(matches!(
        shell
            .present_frame(2, 1)
            .unwrap_or_else(|_| panic!("initial frame executes")),
        UiMountedFrameOutcome::Published(_)
    ));
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::PresentationIndeterminate],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let in_flight = match shell
        .present_frame(4, 3)
        .unwrap_or_else(|_| panic!("successor starts"))
    {
        UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("scripted successor must remain in flight"),
    };
    let indeterminate = match shell.complete_frame_presentation(in_flight, 4) {
        UiMountedFrameOutcome::PresentationIndeterminate(frame) => frame,
        _ => panic!("scripted completion must become indeterminate"),
    };
    let report = indeterminate.report();
    let binding = *report
        .physical_recovery_bindings()
        .first()
        .expect("indeterminate native completion names its recovery binding");
    let exact = worth_ui_host_native::UiNativePhysicalPresentationCorrelation::from_certification(
        report.attempt(),
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        binding,
        1,
    )
    .unwrap();
    let unrelated = worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
        worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas,
        None,
        false,
    );
    let unrelated =
        crate::native_platform::UiNativeApplicationPhysicalProgress::from_host(unrelated);
    let indeterminate =
        match shell.progress_indeterminate_presentation_recovery(indeterminate, &unrelated, 6, 5) {
            WorthUiNativePhysicalPresentationRecovery::Awaiting(frame) => frame,
            WorthUiNativePhysicalPresentationRecovery::Blocked { .. } => {
                panic!("unrelated progress cannot attempt reconstruction")
            }
            WorthUiNativePhysicalPresentationRecovery::Recovered(_) => {
                panic!("unrelated physical progress must not reconstruct")
            }
        };
    assert_eq!(host.presentation_calls(), 2);

    host.push_native_display_presented();
    let exact = worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
        worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery,
        Some(exact),
        false,
    );
    let exact = crate::native_platform::UiNativeApplicationPhysicalProgress::from_host(exact);
    let recovered = shell.progress_indeterminate_presentation_recovery(indeterminate, &exact, 8, 7);
    assert!(matches!(
        recovered,
        WorthUiNativePhysicalPresentationRecovery::Recovered(
            UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Reconciled(_)
        )
    ));
    assert_eq!(host.presentation_calls(), 3);
}

#[cfg(feature = "certification-support")]
#[test]
fn indeterminate_recovery_reconciles_an_uncertain_surface_deregistration() {
    use crate::certification_support::ScriptedSurfaceCompletion;
    use crate::facade::entry::WorthUiNativePhysicalPresentationRecovery;
    use crate::facade::mounted::UiHostSurfaceCancellationOutcome;

    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let mut shell =
        crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host(
            host.clone(),
        )
        .launch_native_surface()
        .expect("native certification shell launches");
    crate::facade::entry::native_application_identity_trace_test_support::
        install_bound_surface_geometry(&mut shell);

    host.push_native_display_presented();
    assert!(matches!(
        shell
            .present_frame(2, 1)
            .unwrap_or_else(|_| panic!("initial frame executes")),
        UiMountedFrameOutcome::Published(_)
    ));
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::PresentationIndeterminate],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let in_flight = match shell
        .present_frame(4, 3)
        .unwrap_or_else(|_| panic!("successor starts"))
    {
        UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("scripted successor must remain in flight"),
    };
    let indeterminate = match shell.complete_frame_presentation(in_flight, 4) {
        UiMountedFrameOutcome::PresentationIndeterminate(frame) => frame,
        _ => panic!("scripted completion must become indeterminate"),
    };
    let report = indeterminate.report();
    let binding = report.physical_recovery_bindings()[0];
    let exact = worth_ui_host_native::UiNativePhysicalPresentationCorrelation::from_certification(
        report.attempt(),
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        binding,
        1,
    )
    .unwrap();
    let exact = worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
        worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery,
        Some(exact),
        false,
    );
    let exact = crate::native_platform::UiNativeApplicationPhysicalProgress::from_host(exact);

    host.return_wrong_next_deregistration_receipt();
    host.push_native_display_presented();
    let recovered = shell.progress_indeterminate_presentation_recovery(indeterminate, &exact, 6, 5);

    assert!(matches!(
        recovered,
        WorthUiNativePhysicalPresentationRecovery::Recovered(
            UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Reconciled(_)
        )
    ));
    assert!(matches!(
        shell.inspect_mounted_frame(UiMountedInspectionRequest::current()),
        UiMountedInspectionReceipt::Available(_)
    ));
    assert_eq!(host.native_registration_count(), 1);
    assert_eq!(host.presentation_calls(), 3);
}
