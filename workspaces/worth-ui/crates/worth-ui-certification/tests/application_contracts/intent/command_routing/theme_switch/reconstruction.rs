use super::*;
use worth_ui::facade::app::WorthUiNativeManagedRebindProgress;
use worth_ui_host_contract::{
    UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationDenial, UiMountedRgba8,
};
use worth_ui_runtime::certification_support::ScriptedSurfaceCompletion;
use worth_ui_runtime::certification_support::{
    ScriptedPresentationAcknowledgement, ScriptedPresentationOutcome,
};
use worth_ui_runtime::facade::mounted::UiMountWorkClass;

#[test]
fn theme_reconstruction_publishes_successor_without_an_intermediate_predecessor() {
    for delayed in [false, true] {
        let host = native_command_host();
        host.set_capabilities(worth_ui_host_native::appearance_capability_report());
        let mut shell = world::application(host.clone())
            .launch_native_surface()
            .unwrap();
        crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
        host.push_native_display_presented();
        assert!(matches!(
            shell.present_frame(10, 1),
            Ok(UiMountedFrameOutcome::Published(_))
        ));
        let surface = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
            UiMountedInspectionReceipt::Available(frame) => {
                frame.presentation().surfaces()[0].semantic_surface()
            }
            _ => panic!("initial Blue publication required"),
        };
        let predecessor = shell.active_theme_binding(surface).unwrap().clone();
        let capability = shell
            .admit_appearance_theme(
                surface,
                &UiThemeDefinitionIdentity::new("theme.command.green").unwrap(),
            )
            .unwrap();
        let request = shell
            .prepare_programmatic_theme_switch(
                surface,
                predecessor.binding_generation(),
                capability,
            )
            .unwrap();
        if delayed {
            host.push_in_flight(
                vec![ScriptedSurfaceCompletion::RejectedBeforeEffects(
                    UiHostSurfacePresentationDenial::ReconstructionRequired,
                )],
                UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
            );
        } else {
            host.push_presentation(ScriptedPresentationOutcome::RejectedBeforeEffects(
                UiHostSurfacePresentationDenial::ReconstructionRequired,
            ));
        }
        // Exactly one accepted presentation may follow the rejected attempt.
        host.push_native_display_presented();
        let calls = host.presentation_calls();
        let outcome = shell
            .begin_managed_theme_switch(
                request,
                UiRebindExecutionPolicy::ordinary(),
                UiRebindExecutionRequest::new(2),
            )
            .unwrap();
        let outcome = if delayed {
            assert!(matches!(
                outcome,
                WorthUiNativeManagedRebindProgress::AwaitingProgress
            ));
            assert_eq!(shell.active_theme_binding(surface), Some(&predecessor));
            let progress =
                worth_ui_native_platform::UiNativeApplicationPhysicalProgress::from_certification(
                    worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
                        worth_ui_host_native::UiNativePhysicalProgressClass::Presentation,
                        host.last_presentation_correlation(),
                    ),
                );
            shell.progress_managed_rebind(&progress).unwrap()
        } else {
            outcome
        };
        assert!(matches!(
            outcome,
            WorthUiNativeManagedRebindProgress::Published(_)
        ));
        assert_eq!(
            host.presentation_calls() - calls,
            2,
            "one denied attempt and one Green reconstruction; delayed={delayed}"
        );
        assert_eq!(
            host.last_surface_colors(),
            vec![UiMountedRgba8::new(16, 144, 48, 255)]
        );
        assert_eq!(
            shell
                .active_theme_binding(surface)
                .unwrap()
                .binding_generation(),
            predecessor.binding_generation() + 1
        );
        assert_eq!(host.pending_presentation_count(), 0);
        assert!(shell.shutdown().host_session_released());
    }
}

#[test]
fn reconstructed_theme_preserves_predecessor_through_settlement_retry_and_shutdown() {
    use worth_ui_host_contract::*;
    for settlement in [
        ReconstructionSettlement::Accept,
        ReconstructionSettlement::Reject,
        ReconstructionSettlement::CancelOnShutdown,
    ] {
        let reject_reconstruction = matches!(settlement, ReconstructionSettlement::Reject);
        let host = native_command_host();
        host.set_capabilities(worth_ui_host_native::appearance_capability_report());
        let mut shell = world::application(host.clone())
            .launch_native_surface()
            .unwrap();
        crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
        host.push_native_display_presented();
        assert!(matches!(
            shell.present_frame(10, 1),
            Ok(UiMountedFrameOutcome::Published(_))
        ));
        let predecessor_presentation = current_presentation(&shell);
        let surface = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
            UiMountedInspectionReceipt::Available(frame) => {
                frame.presentation().surfaces()[0].semantic_surface()
            }
            _ => panic!("initial Blue publication required"),
        };
        let predecessor = shell.active_theme_binding(surface).unwrap().clone();
        let green = UiThemeDefinitionIdentity::new("theme.command.green").unwrap();
        let capability = shell.admit_appearance_theme(surface, &green).unwrap();
        let request = shell
            .prepare_programmatic_theme_switch(
                surface,
                predecessor.binding_generation(),
                capability,
            )
            .unwrap();
        host.push_in_flight(
            vec![ScriptedSurfaceCompletion::RejectedBeforeEffects(
                UiHostSurfacePresentationDenial::ReconstructionRequired,
            )],
            UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
        );
        assert!(matches!(
            shell
                .begin_managed_theme_switch(
                    request,
                    UiRebindExecutionPolicy::ordinary(),
                    UiRebindExecutionRequest::new(2)
                )
                .unwrap(),
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        ));
        let completion = if reject_reconstruction {
            ScriptedSurfaceCompletion::RejectedBeforeEffects(
                UiHostSurfacePresentationDenial::AdapterDeclined,
            )
        } else {
            ScriptedSurfaceCompletion::Presented(ScriptedPresentationAcknowledgement::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                worth_ui_runtime::certification_support::scripted_presentation_epoch(),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                UiHostPresentationCostReport::default(),
            ))
        };
        host.push_in_flight(
            vec![completion],
            UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
        );
        let progress = physical_progress(&host);
        assert!(matches!(
            shell.progress_managed_rebind(&progress).unwrap(),
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        ));
        assert_eq!(shell.active_theme_binding(surface), Some(&predecessor));
        assert!(matches!(
            shell.inspect_mounted_frame(UiMountedInspectionRequest::current()),
            UiMountedInspectionReceipt::Omitted(
                worth_ui::facade::app::UiMountedInspectionOmission::FrameTransitionInFlight
            )
        ));
        assert_eq!(
            host.last_surface_colors(),
            vec![UiMountedRgba8::new(16, 144, 48, 255)],
            "pending reconstruction must already carry Green"
        );
        if matches!(settlement, ReconstructionSettlement::CancelOnShutdown) {
            let shutdown = shell.shutdown();
            assert_eq!(host.cancellation_calls().len(), 1);
            assert!(shutdown.host_session_released());
            assert!(shutdown.intent_resources_empty());
            assert!(shutdown.runtime_service_resources_empty());
            assert!(shutdown.motion_final_census_is_zero());
            continue;
        }
        let progress = physical_progress(&host);
        let outcome = shell.progress_managed_rebind(&progress).unwrap();
        if reject_reconstruction {
            assert!(matches!(
                outcome,
                WorthUiNativeManagedRebindProgress::AwaitingProgress
            ));
            assert_eq!(shell.active_theme_binding(surface), Some(&predecessor));
            assert_eq!(current_presentation(&shell), predecessor_presentation);
            host.push_native_display_presented();
            assert!(matches!(
                shell.retry_managed_rebind(5).unwrap(),
                WorthUiNativeManagedRebindProgress::Published(_)
            ));
        } else {
            assert!(matches!(
                outcome,
                WorthUiNativeManagedRebindProgress::Published(_)
            ));
        }
        assert_eq!(
            shell
                .active_theme_binding(surface)
                .unwrap()
                .binding_generation(),
            predecessor.binding_generation() + 1
        );
        assert_eq!(host.pending_presentation_count(), 0);
        let calls = host.presentation_calls();
        // The ordinary frame accepts unchanged work without repeating reconciliation.
        host.push_native_display_settled_without_effects();
        match shell.present_frame(10, 6) {
            Ok(UiMountedFrameOutcome::Published(receipt)) => {
                assert_eq!(
                    receipt.cost_report().work_class(),
                    UiMountWorkClass::UnchangedReuse
                );
                assert_eq!(
                    receipt.cost_report().appearance().selected_instance_count(),
                    0
                );
            }
            Ok(UiMountedFrameOutcome::Reconciled(_)) => {
                panic!("settled theme repeated reconciliation")
            }
            _ => panic!("post-settlement frame did not complete"),
        }
        assert_eq!(host.presentation_calls(), calls + 1);
        assert!(shell.shutdown().host_session_released());
    }
}

enum ReconstructionSettlement {
    Accept,
    Reject,
    CancelOnShutdown,
}

fn physical_progress(
    host: &worth_ui_runtime::certification_support::ScriptedPresentationHost,
) -> worth_ui_native_platform::UiNativeApplicationPhysicalProgress {
    worth_ui_native_platform::UiNativeApplicationPhysicalProgress::from_certification(
        worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
            worth_ui_host_native::UiNativePhysicalProgressClass::Presentation,
            host.last_presentation_correlation(),
        ),
    )
}
