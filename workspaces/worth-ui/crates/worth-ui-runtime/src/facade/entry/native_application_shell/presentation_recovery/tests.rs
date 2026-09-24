use crate::certification_support::ScriptedPresentationOutcome;
use crate::facade::mounted::{UiHostSurfacePresentationDenial, UiMountedFrameOutcome};
#[cfg(feature = "certification-support")]
use crate::inspection::mounted_frame::{UiMountedInspectionReceipt, UiMountedInspectionRequest};

#[test]
fn host_required_reconstruction_recovers_through_current_mounted_authority() {
    for atlas_first in [false, true] {
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

        host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(
            if atlas_first {
                UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred
            } else {
                UiHostSurfacePresentationDenial::ReconstructionRequired
            },
        ));
        let rejected = shell
            .present_frame(4, 3)
            .unwrap_or_else(|_| panic!("successor frame executes"));
        assert!(matches!(
            rejected,
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
        ));

        if atlas_first {
            host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(
                UiHostSurfacePresentationDenial::ReconstructionRequired,
            ));
        }
        host.push_native_display_presented();
        let recovered = shell
            .resume_frame_presentation(rejected, 6, 5)
            .expect("host-required reconstruction remains available");
        assert!(matches!(
            recovered,
            UiMountedFrameOutcome::Published(_) | UiMountedFrameOutcome::Reconciled(_)
        ));
        assert_eq!(host.presentation_calls(), if atlas_first { 4 } else { 3 });
        assert_eq!(host.pending_presentation_count(), 0);
    }
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
    host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(
        UiHostSurfacePresentationDenial::ExternalTimeout,
    ));
    let rejected = shell
        .present_frame(2, 1)
        .unwrap_or_else(|_| panic!("frame executes"));
    let returned = shell
        .resume_frame_presentation(rejected, 4, 3)
        .expect("unrelated rejection remains an ordinary outcome");
    assert!(matches!(
        returned,
        UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    assert_eq!(host.presentation_calls(), 1);
}

#[test]
fn atlas_ready_resumes_the_rejected_frame_without_rebinding() {
    use crate::certification_support::ScriptedSurfaceCompletion;
    use crate::facade::mounted::UiHostSurfaceCancellationOutcome;

    for asynchronous in [false, true] {
        let host = crate::certification_support::ScriptedPresentationHost::native_display();
        let mut shell = crate::runtime::tests::active_application_session_test_support::
            source_backed_component_app_with_host(host.clone()).launch_native_surface().unwrap();
        crate::facade::entry::native_application_identity_trace_test_support::
            install_bound_surface_geometry(&mut shell);
        host.push_native_display_presented();
        let UiMountedFrameOutcome::Published(predecessor) = shell
            .present_frame(10_000, 0)
            .unwrap_or_else(|_| panic!("initial frame executes"))
        else {
            panic!("initial publication")
        };
        let bindings = predecessor.bindings().to_vec();
        if asynchronous {
            host.push_in_flight(
                vec![ScriptedSurfaceCompletion::RejectedBeforeEffects(
                    UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
                )],
                UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
            );
        } else {
            host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(
                UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
            ));
        }
        let outcome = shell
            .present_frame(10_001, 1)
            .unwrap_or_else(|_| panic!("successor prepares"));
        let outcome = if asynchronous {
            let UiMountedFrameOutcome::InFlight(pending) = outcome else {
                panic!("atlas is pending")
            };
            shell.complete_frame_presentation(pending, 250)
        } else {
            outcome
        };
        assert!(matches!(
            outcome,
            UiMountedFrameOutcome::RejectedBeforeEffects(_)
        ));
        // This successor has unchanged paint: atlas readiness does not invent
        // a NativePaint effect when the accepted work only advances identity.
        host.push_native_display_settled_without_effects();
        let successor = match shell
            .resume_frame_presentation(outcome, 10_001, 250)
            .unwrap()
        {
            UiMountedFrameOutcome::Published(successor) => successor,
            UiMountedFrameOutcome::AdmissionDenied(denial) => {
                panic!("atlas admission: {:?}", denial.denial())
            }
            UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
                panic!("atlas rejection: {:?}", rejected.rejections())
            }
            UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
                panic!("atlas indeterminate: {:?}", frame.report())
            }
            other => panic!("ready atlas outcome: {:?}", std::mem::discriminant(&other)),
        };
        assert_eq!(successor.bindings(), bindings);
        assert_ne!(successor.frame(), predecessor.frame());
        assert_eq!(host.presentation_calls(), 3);
        assert!(host.cancellation_calls().is_empty());
        assert_eq!(host.pending_presentation_count(), 0);
    }
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
