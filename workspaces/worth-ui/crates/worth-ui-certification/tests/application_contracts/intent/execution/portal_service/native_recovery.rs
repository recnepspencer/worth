use worth_ui::facade::{
    app::{
        WorthUiNativeIntentTransition, WorthUiNativeManagedIntentConsequencePublicationOutcome,
        WorthUiNativeManagedRebindProgress, WorthUiNativePredecessorRecovery,
    },
    intent::{
        UiIntentDefinition, UiIntentExecutionAdvanceOutcome, UiIntentRuntimeServiceDestination,
    },
    observation_report::{
        UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationDrain,
        UiHostObservationLoss, UiHostObservationPayload, UiHostObservationPresentationBasis,
        UiHostObservationReport, UiHostObservationSequence, UiHostObservationSequenceRange,
        UiHostObservationTimeBasis, UiHostPointerButton, UiHostPointerButtonTransition,
        UiHostPointerCaptureEpoch, UiHostPointerDeviceKind, UiHostPointerIdentity,
        UiHostProtocolContract,
        UiHostProtocolNegotiation, UiHostSurfacePosition,
        UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
    },
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationOutcome, UiMountedFrameOutcome, UiMountedInspectionReceipt,
    UiMountedInspectionRequest,
};

use super::super::{execution_deadline, execution_reading};
use crate::intent::operability::{build_open_portal_application_with_host, PrimaryIntent};

#[test]
fn native_indeterminate_portal_publication_reconstructs_and_settles_the_predecessor() {
    let host = worth_ui_runtime::certification_support::ScriptedPresentationHost::default();
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(vec![
            worth_ui_host_contract::WorthUiHostCapability::NativePaint,
            worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
            worth_ui_host_contract::WorthUiHostCapability::DpiObservation,
            worth_ui_host_contract::WorthUiHostCapability::FontMetrics,
            worth_ui_host_contract::WorthUiHostCapability::TextIntrinsicMeasurement,
            worth_ui_host_contract::WorthUiHostCapability::TextBaselineMeasurement,
            worth_ui_host_contract::WorthUiHostCapability::PortalAnchorObservation,
            worth_ui_host_contract::WorthUiHostCapability::SemanticFocusPlacement,
        ]),
    );
    host.push_native_display_presented();
    let (application, _) = build_open_portal_application_with_host(host.clone());
    let mut shell = application
        .launch_native_surface()
        .expect("the production native composition root launches");
    let initial = match shell
        .present_frame(10, 1)
        .unwrap_or_else(|_| panic!("the initial native frame executes"))
    {
        UiMountedFrameOutcome::Published(receipt) => receipt,
        _ => panic!("the initial native frame publishes"),
    };
    let inspected = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => frame,
        UiMountedInspectionReceipt::Omitted(denial) => {
            panic!("the published native frame must remain inspectable: {denial:?}")
        }
    };
    let surface = inspected
        .presentation()
        .surfaces()
        .first()
        .expect("the native publication names its real surface");
    let presentation = UiHostObservationPresentationBasis::new(
        surface.host_surface(),
        initial.frame(),
        surface.binding(),
        surface.epoch(),
    );
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let ingress = shell.admit_native_intent_observations(
        definition,
        native_activation_drain(shell.host_session_identity().as_u64(), presentation),
        execution_deadline(20),
    );
    assert!(matches!(
        ingress.transitions(),
        [WorthUiNativeIntentTransition::AttemptPrepared(_)]
    ));
    let transition = match shell.advance_native_intent_executions(execution_reading(1)) {
        UiIntentExecutionAdvanceOutcome::Advanced(report) => {
            let mut transitions = report.into_transitions().into_vec();
            assert_eq!(transitions.len(), 1);
            transitions.pop().unwrap()
        }
        UiIntentExecutionAdvanceOutcome::Stopped(stop) => {
            panic!("native portal provider advance stopped: {stop:?}")
        }
    };
    let handle = transition
        .into_consequence()
        .expect("the native completed portal intent retains its consequence");
    let predecessor_portal = shell.inspect_portal_runtime_for_certification();
    let predecessor_focus = shell.inspect_focus_runtime_for_certification();
    let predecessor_proposals = shell.inspect_service_proposals_for_certification();
    assert!(predecessor_proposals.is_zero());
    host.push_presentation(UiHostSurfacePresentationOutcome::PresentationIndeterminate);
    match shell
        .begin_managed_native_intent_consequence_publication(handle, 40)
        .expect("the managed consequence belongs to this native session")
    {
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Pending => {}
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Published(_) => {
            panic!("the scripted native consequence unexpectedly published")
        }
        WorthUiNativeManagedIntentConsequencePublicationOutcome::NoConsequences(_) => {
            panic!("the portal consequence was lost")
        }
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Stopped(stop) => {
            panic!("the scripted native consequence stopped: {stop:?}")
        }
    }
    host.push_native_display_presented();
    let correlation = host
        .last_presentation_correlation()
        .expect("the indeterminate adapter call retains its exact correlation");
    let grant = worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
        worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery,
        Some(correlation),
        false,
    );
    let progress =
        worth_ui_runtime::native_platform::UiNativeApplicationPhysicalProgress::from_certification(
            grant,
        );
    assert!(matches!(
        shell
            .progress_managed_rebind(&progress)
            .expect("the exact managed recovery remains session-bound"),
        WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(
            WorthUiNativePredecessorRecovery::IntentConsequence
        )
    ));
    assert_eq!(
        shell.inspect_portal_runtime_for_certification(),
        predecessor_portal,
        "reconstruction restores the exact predecessor portal state"
    );
    assert_eq!(
        shell.inspect_focus_runtime_for_certification(),
        predecessor_focus,
        "reconstruction restores the exact predecessor focus state"
    );
    assert_eq!(
        shell.inspect_service_proposals_for_certification(),
        predecessor_proposals,
        "reconstruction releases the proposal transaction"
    );
    let shutdown = shell.shutdown();
    assert!(shutdown.intent_resources_empty());
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert_eq!(shutdown.portal_abandoned_indeterminate_records(), 0);
    assert_eq!(shutdown.focus_abandoned_indeterminate_request(), None);
    assert!(shutdown.host_session_released());
}

#[test]
fn applied_escape_is_retained_while_portal_open_publication_is_in_flight() {
    let host = worth_ui_runtime::certification_support::ScriptedPresentationHost::default();
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(vec![
            worth_ui_host_contract::WorthUiHostCapability::NativePaint,
            worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
            worth_ui_host_contract::WorthUiHostCapability::DpiObservation,
            worth_ui_host_contract::WorthUiHostCapability::FontMetrics,
            worth_ui_host_contract::WorthUiHostCapability::TextIntrinsicMeasurement,
            worth_ui_host_contract::WorthUiHostCapability::TextBaselineMeasurement,
            worth_ui_host_contract::WorthUiHostCapability::PortalAnchorObservation,
            worth_ui_host_contract::WorthUiHostCapability::SemanticFocusPlacement,
        ]),
    );
    host.push_native_display_presented();
    let (application, _) = build_open_portal_application_with_host(host.clone());
    let mut shell = application.launch_native_surface().unwrap();
    assert!(matches!(
        shell
            .present_frame(10, 1)
            .unwrap_or_else(|_| panic!("initial native frame executes")),
        UiMountedFrameOutcome::Published(_)
    ));
    let presentation = super::native_duplicate_dismissal::current_presentation(&shell);
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let ingress = shell.admit_native_intent_observations(
        definition,
        native_activation_drain(shell.host_session_identity().as_u64(), presentation),
        execution_deadline(20),
    );
    assert!(matches!(
        ingress.transitions(),
        [WorthUiNativeIntentTransition::AttemptPrepared(_)]
    ));
    let transition = match shell.advance_native_intent_executions(execution_reading(1)) {
        UiIntentExecutionAdvanceOutcome::Advanced(report) => {
            report.into_transitions().into_vec().pop().unwrap()
        }
        UiIntentExecutionAdvanceOutcome::Stopped(stop) => {
            panic!("portal advance stopped: {stop:?}")
        }
    };
    host.push_in_flight(
        vec![
            worth_ui_runtime::certification_support::ScriptedSurfaceCompletion::Presented(
                worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
                    worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay,
                    worth_ui_runtime::certification_support::scripted_presentation_epoch(),
                    worth_ui_host_contract::UiMountedCompletedEffects::new(vec![
                        worth_ui_host_contract::UiMountedEffectFamily::NativePaint,
                    ]),
                    Default::default(),
                ),
            ),
        ],
        worth_ui_runtime::facade::mounted::UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    assert!(matches!(
        shell
            .begin_managed_native_intent_consequence_publication(
                transition.into_consequence().unwrap(),
                40,
            )
            .unwrap(),
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Pending
    ));
    let escape = super::native_duplicate_dismissal::escape_dismissal(
        &mut shell,
        definition,
        presentation,
        5,
    );
    assert!(matches!(
        shell.begin_managed_portal_dismissal(escape, 41),
        worth_ui::facade::app::WorthUiNativeManagedPortalDismissalOutcome::Retained
    ));
    host.push_native_display_presented();
    let progress =
        worth_ui_runtime::native_platform::UiNativeApplicationPhysicalProgress::from_certification(
            worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
                worth_ui_host_native::UiNativePhysicalProgressClass::Presentation,
                None,
                false,
            ),
        );
    let progress = match shell.progress_managed_rebind(&progress).unwrap() {
        published @ WorthUiNativeManagedRebindProgress::IntentConsequencePublished(_) => published,
        WorthUiNativeManagedRebindProgress::AwaitingProgress => {
            assert_eq!(
                host.native_in_flight_count(),
                0,
                "physical surface completion must be consumed before another class is awaited"
            );
            let text_progress = worth_ui_runtime::native_platform::UiNativeApplicationPhysicalProgress::from_certification(
                worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
                    worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas,
                    None,
                    false,
                ),
            );
            shell.progress_managed_rebind(&text_progress).unwrap()
        }
        WorthUiNativeManagedRebindProgress::RecoveryBlocked(_) => {
            panic!("ordinary in-flight open entered recovery")
        }
        WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(_) => {
            panic!("ordinary in-flight open recovered its predecessor")
        }
        WorthUiNativeManagedRebindProgress::PortalDismissed(_) => {
            panic!("open completion skipped its publication")
        }
        WorthUiNativeManagedRebindProgress::Published(_) => {
            panic!("portal open was misreported as an ordinary rebind publication")
        }
        WorthUiNativeManagedRebindProgress::Unrelated => {
            panic!("matching native completion was unrelated")
        }
        WorthUiNativeManagedRebindProgress::Stopped(_) => panic!("open completion stopped"),
    };
    assert!(matches!(
        progress,
        WorthUiNativeManagedRebindProgress::IntentConsequencePublished(_)
    ));
    match shell.continue_retained_portal_dismissal_after_managed_intent(42) {
        worth_ui::facade::app::WorthUiNativeManagedPortalDismissalOutcome::Published(_) => {}
        worth_ui::facade::app::WorthUiNativeManagedPortalDismissalOutcome::Ignored => {
            panic!("published open lost its retained Escape")
        }
        worth_ui::facade::app::WorthUiNativeManagedPortalDismissalOutcome::Retained => {
            panic!("completed open still occupied the managed slot")
        }
        worth_ui::facade::app::WorthUiNativeManagedPortalDismissalOutcome::Pending => {
            panic!("scripted immediate dismissal remained pending")
        }
        worth_ui::facade::app::WorthUiNativeManagedPortalDismissalOutcome::Stopped(stop) => {
            panic!("retained Escape stopped: {stop:?}")
        }
    }
    assert_eq!(
        shell
            .inspect_portal_runtime_for_certification()
            .active_portals(),
        1
    );
    assert_eq!(
        shell
            .inspect_portal_runtime_for_certification()
            .closing_portals(),
        1
    );
    assert!(shell
        .inspect_service_proposals_for_certification()
        .is_zero());
    let shutdown = shell.shutdown();
    assert!(shutdown.intent_resources_empty());
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert!(shutdown.host_session_released());
}

pub(super) fn native_activation_drain(
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
) -> UiHostObservationDrain {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let viewport = UiHostObservationSequence::new(1);
    let scale = UiHostObservationSequence::new(2);
    let pressed = UiHostObservationSequence::new(3);
    let released = UiHostObservationSequence::new(4);
    let position = UiHostSurfacePosition::viewport_logical(
        18 * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
        20 * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
    );
    let report = |sequence, transition| {
        UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            UiHostObservationPayload::PointerButton {
                pointer: UiHostPointerIdentity::new(1),
                capture_epoch: UiHostPointerCaptureEpoch::new(1),
                button: UiHostPointerButton::Primary,
                transition,
                position,
            },
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .expect("native pointer reports carry an explicit device kind")
    };
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(viewport, released),
        loss: UiHostObservationLoss::Complete,
        reports: vec![
            UiHostObservationReport::new(
                viewport,
                UiHostObservationTimeBasis::HostMonotonicMillis(viewport.value()),
                UiHostObservationPayload::Viewport {
                    width_subpixels: 800 * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
                    height_subpixels: 600 * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
                },
            ),
            UiHostObservationReport::new(
                scale,
                UiHostObservationTimeBasis::HostMonotonicMillis(scale.value()),
                UiHostObservationPayload::DeviceScale { micros: 1_000_000 },
            ),
            report(pressed, UiHostPointerButtonTransition::Pressed),
            report(released, UiHostPointerButtonTransition::Released),
        ],
    })
    .expect("the native activation batch satisfies the host protocol");
    UiHostObservationDrain::bounded(vec![batch])
        .expect("one two-report native activation is mechanically bounded")
}
