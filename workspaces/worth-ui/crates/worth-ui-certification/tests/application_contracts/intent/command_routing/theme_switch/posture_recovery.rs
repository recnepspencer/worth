use super::*;
use worth_ui::facade::app::WorthUiNativeManagedRebindProgress;
use worth_ui_host_contract::{UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationOutcome};
use worth_ui_host_native::{UiNativePhysicalProgressClass, UiNativePhysicalProgressGrant};
use worth_ui_runtime::certification_support::ScriptedSurfaceCompletion;

#[test]
fn indeterminate_posture_recovers_predecessor_before_a_fresh_publication_or_shutdown() {
    for (delayed, shutdown_pending) in [(false, false), (true, false), (false, true)] {
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
        let predecessor = current_presentation(&shell);
        let prior_correlation = host.last_presentation_correlation().unwrap();
        let definition = || {
            UiIntentDefinition::<AdvanceStatus>::runtime_service(
                UiIntentRuntimeServiceDestination::InvokeCommand,
            )
        };
        let ingress = shell.admit_native_intent_observations(
            definition(),
            shortcut_drain(shell.host_session_identity().as_u64(), predecessor, false),
            crate::intent::execution::execution_deadline(20),
        );
        let mut transitions = ingress.into_transitions().into_vec();
        assert_eq!(transitions.len(), 1);
        let WorthUiNativeIntentTransition::AttemptPrepared(prepared) = transitions.remove(0) else {
            panic!("the real shortcut must admit its command");
        };
        if delayed {
            host.push_in_flight(
                vec![ScriptedSurfaceCompletion::PresentationIndeterminate],
                UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
            );
        } else {
            host.push_presentation(UiHostSurfacePresentationOutcome::PresentationIndeterminate);
        }
        assert!(matches!(
            shell.begin_managed_native_intent_posture_publication(prepared.into_posture(), 21),
            Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Pending)
        ));
        let correlation = host.last_presentation_correlation().unwrap();
        let progress = |class, correlation| {
            worth_ui_native_platform::UiNativeApplicationPhysicalProgress::from_certification(
                UiNativePhysicalProgressGrant::from_certification(class, Some(correlation), false),
            )
        };
        if delayed {
            assert!(matches!(
                shell
                    .progress_managed_rebind(&progress(
                        UiNativePhysicalProgressClass::Presentation,
                        correlation
                    ))
                    .unwrap(),
                WorthUiNativeManagedRebindProgress::AwaitingProgress
            ));
        }
        assert_eq!(current_presentation(&shell), predecessor);
        if !shutdown_pending {
            let calls = host.presentation_calls();
            assert!(matches!(
                shell
                    .progress_managed_rebind(&progress(
                        UiNativePhysicalProgressClass::PresentationRecovery,
                        prior_correlation
                    ))
                    .unwrap(),
                WorthUiNativeManagedRebindProgress::AwaitingProgress
            ));
            assert_eq!(host.presentation_calls(), calls);
            host.push_native_display_presented();
            let recovered = shell
                .progress_managed_rebind(&progress(
                    UiNativePhysicalProgressClass::PresentationRecovery,
                    correlation,
                ))
                .unwrap();
            let WorthUiNativeManagedRebindProgress::RebindRecovered(receipt) = recovered else {
                panic!("posture recovery must never claim successor publication");
            };
            assert!(receipt.predecessor_remains_current());
            assert_eq!(receipt.affected_bindings(), &[correlation.binding()]);
            drop(receipt);
            assert_eq!(
                host.last_surface_colors(),
                vec![worth_ui_host_contract::UiMountedRgba8::new(
                    24, 48, 160, 255
                )]
            );
            let ingress = shell.admit_native_intent_observations(
                definition(),
                shortcut_drain_at(
                    shell.host_session_identity().as_u64(),
                    current_presentation(&shell),
                    false,
                    2,
                ),
                crate::intent::execution::execution_deadline(25),
            );
            let mut transitions = ingress.into_transitions().into_vec();
            assert_eq!(transitions.len(), 1);
            let WorthUiNativeIntentTransition::AttemptPrepared(prepared) = transitions.remove(0)
            else {
                panic!("recovery must release admission for a fresh command");
            };
            host.push_native_display_settled_without_effects();
            let outcome = shell
                .begin_managed_native_intent_posture_publication(prepared.into_posture(), 26)
                .unwrap();
            let WorthUiNativeManagedIntentPosturePublicationOutcome::Published(receipt) = outcome
            else {
                panic!("fresh posture must publish after predecessor recovery");
            };
            assert!(shell
                .issue_theme_switch_origin_from_publication(&receipt)
                .is_ok());
            drop(receipt);
        }
        assert_eq!(host.pending_presentation_count(), 0);
        let shutdown = shell.shutdown();
        assert!(shutdown.host_session_released());
        assert!(shutdown.intent_resources_empty());
        assert!(shutdown.runtime_service_resources_empty());
    }
}
