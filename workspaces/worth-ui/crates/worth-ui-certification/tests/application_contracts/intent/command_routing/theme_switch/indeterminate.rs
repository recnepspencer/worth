use super::*;
use worth_ui::facade::app::WorthUiNativeManagedRebindProgress;
use worth_ui_host_contract::{UiHostSurfaceCancellationOutcome, UiMountedRgba8};
use worth_ui_host_native::{UiNativePhysicalProgressClass, UiNativePhysicalProgressGrant};
use worth_ui_runtime::certification_support::ScriptedSurfaceCompletion;
use worth_ui_runtime::certification_support::{
    ScriptedPresentationAcknowledgement, ScriptedPresentationOutcome,
};

#[test]
fn managed_theme_indeterminacy_retains_recovery_then_allows_a_fresh_successor() {
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
        let prior_correlation = host.last_presentation_correlation().unwrap();
        let surface = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
            UiMountedInspectionReceipt::Available(frame) => {
                frame.presentation().surfaces()[0].semantic_surface()
            }
            _ => panic!("initial Blue publication required"),
        };
        let blue = shell.active_theme_binding(surface).unwrap().clone();
        let green = UiThemeDefinitionIdentity::new("theme.command.green").unwrap();
        let capability = shell.admit_appearance_theme(surface, &green).unwrap();
        let request = shell
            .prepare_programmatic_theme_switch(
                surface,
                blue.binding_generation(),
                capability.clone(),
            )
            .unwrap();
        if delayed {
            host.push_in_flight(
                vec![ScriptedSurfaceCompletion::PresentationIndeterminate],
                UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
            );
        } else {
            host.push_presentation(ScriptedPresentationOutcome::PresentationIndeterminate);
        }
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
        let correlation = host.last_presentation_correlation().unwrap();
        let progress = |class, correlation| {
            worth_ui_native_platform::UiNativeApplicationPhysicalProgress::from_certification(
                UiNativePhysicalProgressGrant::from_certification(class, correlation),
            )
        };
        if delayed {
            assert!(matches!(
                shell
                    .progress_managed_rebind(&progress(
                        UiNativePhysicalProgressClass::Presentation,
                        Some(correlation)
                    ))
                    .unwrap(),
                WorthUiNativeManagedRebindProgress::AwaitingProgress
            ));
        }
        assert_eq!(shell.active_theme_binding(surface), Some(&blue));
        assert!(matches!(
            shell.prepare_programmatic_theme_switch(surface, blue.binding_generation(), capability),
            Err(UiProgrammaticThemeSwitchPreparationDenial::ManagedRebindAlreadyInFlight)
        ));
        let calls = host.presentation_calls();
        assert!(matches!(
            shell
                .progress_managed_rebind(&progress(UiNativePhysicalProgressClass::TextAtlas, None))
                .unwrap(),
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        ));
        assert_eq!(
            host.presentation_calls(),
            calls,
            "unrelated physical progress cannot authorize reconstruction"
        );
        assert!(matches!(
            shell
                .progress_managed_rebind(&progress(
                    UiNativePhysicalProgressClass::PresentationRecovery,
                    Some(prior_correlation)
                ))
                .unwrap(),
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        ));
        assert_eq!(
            host.presentation_calls(),
            calls,
            "a different attempt cannot authorize recovery"
        );
        host.push_native_display_presented();
        let outcome = shell
            .progress_managed_rebind(&progress(
                UiNativePhysicalProgressClass::PresentationRecovery,
                Some(correlation),
            ))
            .unwrap();
        let WorthUiNativeManagedRebindProgress::RebindRecovered(receipt) = outcome else {
            panic!("matching recovery must restore the predecessor")
        };
        assert!(receipt.predecessor_remains_current());
        assert_eq!(receipt.affected_bindings(), &[correlation.binding()]);
        drop(receipt);
        assert_eq!(shell.active_theme_binding(surface), Some(&blue));
        assert_eq!(
            host.last_surface_colors(),
            vec![UiMountedRgba8::new(24, 48, 160, 255)],
            "recovery settles the authoritative Blue predecessor"
        );
        let capability = shell.admit_appearance_theme(surface, &green).unwrap();
        let request = shell
            .prepare_programmatic_theme_switch(surface, blue.binding_generation(), capability)
            .unwrap();
        host.push_native_display_presented();
        assert!(matches!(
            shell
                .begin_managed_theme_switch(
                    request,
                    UiRebindExecutionPolicy::ordinary(),
                    UiRebindExecutionRequest::new(6)
                )
                .unwrap(),
            WorthUiNativeManagedRebindProgress::Published(_)
        ));
        assert_eq!(
            host.last_surface_colors(),
            vec![UiMountedRgba8::new(16, 144, 48, 255)]
        );
        assert_eq!(host.pending_presentation_count(), 0);
        let shutdown = shell.shutdown();
        assert!(shutdown.host_session_released());
        assert!(shutdown.intent_resources_empty());
        assert!(shutdown.runtime_service_resources_empty());
    }
}

#[test]
fn managed_rebind_recovery_survives_interrupted_reconstruction_and_shutdown() {
    use worth_ui_host_contract::*;
    for obstacle in [
        RecoveryObstacle::Rejection,
        RecoveryObstacle::InFlight,
        RecoveryObstacle::Indeterminate,
        RecoveryObstacle::Shutdown,
    ] {
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
            _ => panic!("Blue predecessor required"),
        };
        let blue = shell.active_theme_binding(surface).unwrap().clone();
        let capability = shell
            .admit_appearance_theme(
                surface,
                &UiThemeDefinitionIdentity::new("theme.command.green").unwrap(),
            )
            .unwrap();
        let request = shell
            .prepare_programmatic_theme_switch(surface, blue.binding_generation(), capability)
            .unwrap();
        host.push_presentation(ScriptedPresentationOutcome::PresentationIndeterminate);
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
        let correlation = host.last_presentation_correlation();
        match obstacle {
            RecoveryObstacle::Rejection => host.push_rejected(),
            RecoveryObstacle::Indeterminate => {
                host.push_presentation(ScriptedPresentationOutcome::PresentationIndeterminate)
            }
            RecoveryObstacle::InFlight | RecoveryObstacle::Shutdown => host.push_in_flight(
                vec![ScriptedSurfaceCompletion::Presented(
                    ScriptedPresentationAcknowledgement::new(
                        UiHostSurfacePresentationMode::NativeDisplay,
                        worth_ui_runtime::certification_support::scripted_presentation_epoch(),
                        UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                        UiHostPresentationCostReport::default(),
                    ),
                )],
                UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
            ),
        }
        let progress = |class, correlation| {
            worth_ui_native_platform::UiNativeApplicationPhysicalProgress::from_certification(
                UiNativePhysicalProgressGrant::from_certification(class, correlation),
            )
        };
        let outcome = shell
            .progress_managed_rebind(&progress(
                UiNativePhysicalProgressClass::PresentationRecovery,
                correlation,
            ))
            .unwrap();
        if matches!(obstacle, RecoveryObstacle::Rejection) {
            assert!(matches!(
                outcome,
                WorthUiNativeManagedRebindProgress::RecoveryBlocked(_)
            ));
        } else {
            assert!(matches!(
                outcome,
                WorthUiNativeManagedRebindProgress::AwaitingProgress
            ));
        }
        assert_eq!(shell.active_theme_binding(surface), Some(&blue));
        if matches!(obstacle, RecoveryObstacle::Shutdown) {
            let shutdown = shell.shutdown();
            assert_eq!(host.cancellation_calls().len(), 1);
            assert!(shutdown.host_session_released());
            assert!(shutdown.intent_resources_empty());
            assert!(shutdown.runtime_service_resources_empty());
            continue;
        }
        let recovery_progress = match obstacle {
            RecoveryObstacle::Rejection => {
                host.push_native_display_presented();
                progress(UiNativePhysicalProgressClass::TextAtlas, None)
            }
            RecoveryObstacle::Indeterminate => {
                let calls = host.presentation_calls();
                assert!(matches!(
                    shell
                        .progress_managed_rebind(&progress(
                            UiNativePhysicalProgressClass::PresentationRecovery,
                            correlation
                        ))
                        .unwrap(),
                    WorthUiNativeManagedRebindProgress::AwaitingProgress
                ));
                assert_eq!(
                    host.presentation_calls(),
                    calls,
                    "a new uncertain reconstruction requires its own correlation"
                );
                host.push_native_display_presented();
                progress(
                    UiNativePhysicalProgressClass::PresentationRecovery,
                    host.last_presentation_correlation(),
                )
            }
            RecoveryObstacle::InFlight => progress(
                UiNativePhysicalProgressClass::Presentation,
                host.last_presentation_correlation(),
            ),
            RecoveryObstacle::Shutdown => unreachable!(),
        };
        assert!(matches!(
            shell.progress_managed_rebind(&recovery_progress).unwrap(),
            WorthUiNativeManagedRebindProgress::RebindRecovered(_)
        ));
        assert_eq!(shell.active_theme_binding(surface), Some(&blue));
        assert_eq!(
            host.last_surface_colors(),
            vec![UiMountedRgba8::new(24, 48, 160, 255)]
        );
        assert!(shell.shutdown().host_session_released());
    }
}

enum RecoveryObstacle {
    Rejection,
    InFlight,
    Indeterminate,
    Shutdown,
}
