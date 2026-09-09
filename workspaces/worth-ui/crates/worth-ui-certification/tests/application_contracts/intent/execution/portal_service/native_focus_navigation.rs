use worth_ui::facade::{
    app::{WorthUiNativeIntentTransition, WorthUiNativeManagedIntentConsequencePublicationOutcome},
    intent::{
        UiIntentDefinition, UiIntentExecutionAdvanceOutcome, UiIntentRuntimeServiceDestination,
    },
    observation_report::{
        UiHostKey, UiHostKeyTransition, UiHostKeyboardModifiers, UiHostObservationBatch,
        UiHostObservationBatchInput, UiHostObservationDrain, UiHostObservationLoss,
        UiHostObservationPayload, UiHostObservationPresentationBasis, UiHostObservationReport,
        UiHostObservationSequence, UiHostObservationSequenceRange, UiHostObservationTimeBasis,
        UiHostProtocolContract, UiHostProtocolNegotiation,
    },
};
use worth_ui_runtime::facade::mounted::UiMountedFrameOutcome;

use super::super::{execution_deadline, execution_reading};
use super::native_recovery::native_activation_drain;
use crate::intent::operability::{
    build_open_portal_two_focus_application_with_host, PrimaryIntent,
};

#[test]
fn portal_shift_tab_navigation_rebinds_enter_to_the_newly_focused_recipient() {
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
    let (application, _) = build_open_portal_two_focus_application_with_host(host.clone());
    let mut shell = application
        .launch_native_surface()
        .expect("the production native composition root launches");
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    assert!(matches!(
        shell
            .present_frame(10, 1)
            .unwrap_or_else(|_| panic!("initial frame executes")),
        UiMountedFrameOutcome::Published(_)
    ));

    let initial_presentation = super::native_duplicate_dismissal::current_presentation(&shell);
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let ingress = shell.admit_native_intent_observations(
        definition,
        native_activation_drain(shell.host_session_identity().as_u64(), initial_presentation),
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
    host.push_native_display_presented();
    assert!(matches!(
        shell
            .begin_managed_native_intent_consequence_publication(
                transition.into_consequence().unwrap(),
                40,
            )
            .expect("the managed Portal consequence belongs to this native session"),
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Published(_)
    ));

    let presentation = super::native_duplicate_dismissal::current_presentation(&shell);
    let before = host
        .input_recipient()
        .expect("Portal initial focus installs its production input affinity");
    let before_focus = shell
        .inspect_focus_runtime_for_certification()
        .current_participant()
        .expect("Portal initial focus is committed");
    assert_eq!(before.mounted_instance().diagnostic_value(), before_focus);

    let tab = keyboard_drain(
        shell.host_session_identity().as_u64(),
        presentation,
        before,
        5,
        UiHostKey::Tab,
        UiHostKeyboardModifiers::new(false, false, true, false, false),
    );
    let tab_ingress =
        shell.admit_native_intent_observations(definition, tab, execution_deadline(50));
    assert!(tab_ingress.interaction_stops().is_empty());
    let after_snapshot = shell.inspect_focus_runtime_for_certification();
    let after_focus = after_snapshot.current_participant().unwrap_or_else(|| {
        panic!("Tab commits a current semantic focus participant: {after_snapshot:?}")
    });
    assert_ne!(
        after_focus, before_focus,
        "the fixture must exercise a real focus move"
    );
    let after = host
        .input_recipient()
        .expect("Tab installs input affinity for its committed focus target");
    assert_eq!(after.mounted_instance().diagnostic_value(), after_focus);
    assert_ne!(
        after, before,
        "Tab cannot leave predecessor affinity active"
    );

    let enter = keyboard_drain(
        shell.host_session_identity().as_u64(),
        presentation,
        after,
        6,
        UiHostKey::Enter,
        UiHostKeyboardModifiers::default(),
    );
    let enter_ingress =
        shell.admit_native_intent_observations(definition, enter, execution_deadline(60));
    assert!(
        enter_ingress.interaction_stops().is_empty(),
        "Enter with the successor affinity must reach the newly focused recipient"
    );

    let shutdown = shell.shutdown();
    assert!(shutdown.runtime_service_resources_empty());
}

fn keyboard_drain(
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
    recipient: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    sequence: u64,
    key: UiHostKey,
    modifiers: UiHostKeyboardModifiers,
) -> UiHostObservationDrain {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let sequence = UiHostObservationSequence::new(sequence);
    let report = UiHostObservationReport::new(
        sequence,
        UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
        UiHostObservationPayload::Keyboard {
            logical_key: key,
            physical_key: Some(key),
            modifiers,
            transition: UiHostKeyTransition::Pressed { repeat: false },
        },
    )
    .with_input_affinity(
        worth_ui_host_contract::UiHostInputRecipientAffinityReceipt::at_event_time(
            recipient,
            presentation,
        ),
    );
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![report],
    })
    .expect("the keyboard batch satisfies the production host protocol");
    UiHostObservationDrain::bounded(vec![batch]).expect("one keyboard report is bounded")
}
