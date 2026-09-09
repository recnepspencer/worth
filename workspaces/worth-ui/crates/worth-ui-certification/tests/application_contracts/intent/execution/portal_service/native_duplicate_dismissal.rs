use worth_ui::facade::{
    app::{
        WorthUiNativeApplicationShell, WorthUiNativeIntentTransition,
        WorthUiNativeManagedIntentConsequencePublicationOutcome,
        WorthUiNativeManagedPortalDismissalOutcome, WorthUiNativeManagedRebindProgress,
    },
    intent::{
        UiIntentDefinition, UiIntentExecutionAdvanceOutcome, UiIntentRuntimeServiceDestination,
        UiRuntimeServiceDefinitionDestination,
    },
    observation_report::{
        UiHostKey, UiHostKeyTransition, UiHostKeyboardModifiers, UiHostObservationBatch,
        UiHostObservationBatchInput, UiHostObservationDrain, UiHostObservationLoss,
        UiHostObservationPayload, UiHostObservationPresentationBasis, UiHostObservationReport,
        UiHostObservationSequence, UiHostObservationSequenceRange, UiHostObservationTimeBasis,
        UiHostProtocolContract, UiHostProtocolNegotiation,
    },
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationOutcome, UiMountedFrameOutcome, UiMountedInspectionReceipt,
    UiMountedInspectionRequest,
};

use super::super::{execution_deadline, execution_reading};
use super::native_recovery::native_activation_drain;
use crate::intent::{
    operability::{build_open_portal_application_with_host, PrimaryIntent},
    runtime_services_kit::{NativeRuntimeServiceEvidence, RuntimeServiceSemanticOutcome},
};

#[test]
fn duplicate_escape_survives_indeterminate_portal_dismissal_recovery() {
    let evidence = run_native_runtime_service_scenario();
    assert!(evidence.semantic.portal_was_visible);
    assert!(evidence.semantic.focus_was_placed);
    assert!(evidence.semantic.dismissal_closed_only_top);
    assert!(evidence.semantic.focus_restored_to_previous);
    assert!(evidence.semantic.duplicate_was_idempotent);
    assert!(evidence.semantic.proposals_are_zero);
    assert!(evidence.semantic.terminal_resources_are_zero);
    assert!(evidence.indeterminate_effect_retained);
    assert!(evidence.reconciled_from_exact_host_truth);
    assert!(evidence.predecessor_was_reconstructed);
}

pub(crate) fn run_native_runtime_service_scenario() -> NativeRuntimeServiceEvidence {
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
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    match shell
        .present_frame(10, 1)
        .unwrap_or_else(|denial| match denial {
            worth_ui::facade::app::WorthUiMountedFrameExecutionStop::PublicationLease(reason) => {
                panic!("the initial native frame lease failed: {reason:?}")
            }
            worth_ui::facade::app::WorthUiMountedFrameExecutionStop::HostMeasurement(reason) => {
                panic!("the initial native frame measurement failed: {reason:?}")
            }
            worth_ui::facade::app::WorthUiMountedFrameExecutionStop::HostMeasurementTransition(
                reason,
            ) => panic!("the initial native frame measurement transition failed: {reason:?}"),
            worth_ui::facade::app::WorthUiMountedFrameExecutionStop::OccurrenceGeometry(reason) => {
                panic!("the initial native frame geometry failed: {reason:?}")
            }
            worth_ui::facade::app::WorthUiMountedFrameExecutionStop::Preparation(reason) => {
                panic!("the initial native frame preparation failed: {reason:?}")
            }
            worth_ui::facade::app::WorthUiMountedFrameExecutionStop::FrameworkTransition(_) => {
                panic!("the initial native frame transition failed")
            }
        }) {
        UiMountedFrameOutcome::Published(_) => {}
        _ => panic!("the initial native frame publishes"),
    }
    let pre_portal_focus = shell.inspect_focus_runtime_for_certification();
    assert_eq!(pre_portal_focus.current_participant(), None);
    let presentation = current_presentation(&shell);
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
            panic!("native portal provider advance stopped: {stop:?}")
        }
    };
    let handle = transition
        .into_consequence()
        .expect("the completed portal intent retains its consequence");
    host.push_native_display_presented();
    let opened = match shell
        .begin_managed_native_intent_consequence_publication(handle, 40)
        .expect("the managed consequence belongs to this native session")
    {
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Published(receipt) => receipt,
        _ => panic!("the portal consequence must publish before dismissal"),
    };
    drop(opened);
    let open_presentation = current_presentation(&shell);
    let initial_focus_request = host
        .last_focus_placement()
        .expect("portal opening issues the negotiated native focus mechanic");
    assert_eq!(initial_focus_request.presentation(), open_presentation);
    assert!(shell
        .inspect_focus_runtime_for_certification()
        .current_participant()
        .is_some());
    let portal_was_visible = shell
        .inspect_portal_runtime_for_certification()
        .visible_portals()
        == 1;
    let focus_was_placed = shell
        .inspect_focus_runtime_for_certification()
        .current_participant()
        .is_some();

    let first = escape_dismissal(&mut shell, definition, open_presentation, 5);
    let duplicate = escape_dismissal(&mut shell, definition, open_presentation, 7);
    host.push_presentation(UiHostSurfacePresentationOutcome::PresentationIndeterminate);
    assert!(matches!(
        shell.begin_managed_portal_dismissal(first, 50),
        WorthUiNativeManagedPortalDismissalOutcome::Pending
    ));
    let correlation = host
        .last_presentation_correlation()
        .expect("the indeterminate dismissal retains its exact correlation");

    assert!(matches!(
        shell.begin_managed_portal_dismissal(duplicate, 51),
        WorthUiNativeManagedPortalDismissalOutcome::Pending
    ));
    let indeterminate_effect_retained = host.last_presentation_correlation() == Some(correlation);
    assert!(
        indeterminate_effect_retained,
        "duplicate dismissal cannot replace the retained indeterminate effect correlation"
    );

    host.push_native_display_presented();
    host.push_native_display_presented();
    let grant = worth_ui_host_native::UiNativePhysicalProgressGrant::from_certification(
        worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery,
        Some(correlation),
        false,
    );
    let progress =
        worth_ui_runtime::native_platform::UiNativeApplicationPhysicalProgress::from_certification(
            grant,
        );
    let reconciled_from_exact_host_truth = matches!(
        shell
            .progress_managed_rebind(&progress)
            .expect("the exact managed recovery remains session-bound"),
        WorthUiNativeManagedRebindProgress::PortalDismissed(_)
    );
    assert!(reconciled_from_exact_host_truth);
    let predecessor_was_reconstructed = host.reconstruction_portal_overlay_counts().contains(&1);
    assert!(
        predecessor_was_reconstructed,
        "dismissal recovery must reconstruct the visible predecessor overlay before replay"
    );
    let portal = shell.inspect_portal_runtime_for_certification();
    assert_eq!(portal.active_portals(), 1);
    assert_eq!(portal.open_portals(), 0);
    assert_eq!(portal.visible_portals(), 0);
    assert_eq!(portal.closing_portals(), 1);
    let restored_focus = shell.inspect_focus_runtime_for_certification();
    assert_eq!(
        restored_focus.current_participant(),
        pre_portal_focus.current_participant(),
        "dismissal recovery restores the exact pre-portal semantic focus posture",
    );
    assert_eq!(restored_focus.pending_portal_transitions(), 0);
    assert!(shell
        .inspect_service_proposals_for_certification()
        .is_zero());
    let duplicate_was_idempotent = portal.committed_requests() == 2
        && portal.active_portals() == 1
        && portal.closing_portals() == 1;
    let proposals_are_zero = shell
        .inspect_service_proposals_for_certification()
        .is_zero();
    let shutdown = shell.shutdown();
    assert!(shutdown.intent_resources_empty());
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert_eq!(shutdown.portal_abandoned_indeterminate_records(), 0);
    assert_eq!(shutdown.focus_abandoned_indeterminate_request(), None);
    let host_session_released = shutdown.host_session_released();
    assert!(host_session_released);
    NativeRuntimeServiceEvidence {
        semantic: RuntimeServiceSemanticOutcome {
            portal_was_visible,
            focus_was_placed,
            dismissal_closed_only_top: portal.active_portals() == 1
                && portal.open_portals() == 0
                && portal.closing_portals() == 1,
            focus_restored_to_previous: restored_focus.current_participant()
                == pre_portal_focus.current_participant(),
            duplicate_was_idempotent,
            proposals_are_zero,
            terminal_resources_are_zero: shutdown.runtime_service_resources_empty()
                && host_session_released,
        },
        indeterminate_effect_retained,
        reconciled_from_exact_host_truth,
        predecessor_was_reconstructed,
    }
}

pub(super) fn current_presentation(
    shell: &WorthUiNativeApplicationShell,
) -> UiHostObservationPresentationBasis {
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
    UiHostObservationPresentationBasis::new(
        surface.host_surface(),
        inspected.frame(),
        surface.binding(),
        surface.epoch(),
    )
}

pub(super) fn escape_dismissal(
    shell: &mut WorthUiNativeApplicationShell,
    definition: UiIntentDefinition<PrimaryIntent, UiRuntimeServiceDefinitionDestination>,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
) -> worth_ui::facade::interaction::UiDismissInteraction {
    let ingress = shell.admit_native_intent_observations(
        definition,
        escape_drain(
            shell.host_session_identity().as_u64(),
            presentation,
            sequence,
        ),
        execution_deadline(60 + sequence),
    );
    assert!(ingress.transitions().is_empty());
    assert!(
        ingress.interaction_stops().is_empty(),
        "Escape ingress stopped: {:?}",
        ingress
            .interaction_stops()
            .iter()
            .map(|stop| match stop {
                worth_ui::facade::app::WorthUiNativeInteractionIngressStop::Quarantined(stop) =>
                    format!("quarantined:{:?}", stop.quarantine()),
                worth_ui::facade::app::WorthUiNativeInteractionIngressStop::Denied(stop) =>
                    format!("denied:{:?}", stop.denial()),
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(ingress.dismissals().len(), 1);
    ingress.dismissals()[0]
}

fn escape_drain(
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
) -> UiHostObservationDrain {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let focus_sequence = UiHostObservationSequence::new(sequence);
    let escape_sequence = UiHostObservationSequence::new(sequence + 1);
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(focus_sequence, escape_sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![
            UiHostObservationReport::new(
                focus_sequence,
                UiHostObservationTimeBasis::HostMonotonicMillis(focus_sequence.value()),
                UiHostObservationPayload::WindowFocus {
                    surface: presentation.host_surface(),
                    focused: true,
                },
            ),
            UiHostObservationReport::new(
                escape_sequence,
                UiHostObservationTimeBasis::HostMonotonicMillis(escape_sequence.value()),
                UiHostObservationPayload::Keyboard {
                    logical_key: UiHostKey::Escape,
                    physical_key: Some(UiHostKey::Escape),
                    modifiers: UiHostKeyboardModifiers::default(),
                    transition: UiHostKeyTransition::Pressed { repeat: false },
                },
            ),
        ],
    })
    .expect("the Escape batch satisfies the host protocol");
    UiHostObservationDrain::bounded(vec![batch]).expect("one Escape report is mechanically bounded")
}
