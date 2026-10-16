//! Normalized raw key data enters the existing native admission door with real frame/session truth.
use super::*;
use worth_ui::facade::{
    app::{UiMountedInspectionReceipt, UiMountedInspectionRequest},
    observation_report::*,
};
impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn dispatch_query_action_for_test(&mut self) {
        let Some(probe) = self.query_probe.clone() else {
            return;
        };
        if probe.borrow().action_dispatched
            || !probe
                .borrow()
                .presented
                .iter()
                .any(|value| value.native_value() == Some("Scheduled"))
        {
            return;
        }
        if !self
            .query_lifecycle
            .as_ref()
            .is_some_and(|query| query.accepts_command())
        {
            return;
        }
        let mut shell = self.take_runtime_shell();
        let inspected = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
            UiMountedInspectionReceipt::Available(frame) => frame,
            UiMountedInspectionReceipt::Omitted(denial) => {
                panic!("actual presented frame required: {denial:?}")
            }
        };
        let surface = inspected
            .presentation()
            .surfaces()
            .first()
            .expect("real native surface");
        let presentation = UiHostObservationPresentationBasis::new(
            surface.host_surface(),
            inspected.frame(),
            surface.binding(),
            surface.epoch(),
        );
        let protocol = match UiHostProtocolContract::current().negotiate() {
            UiHostProtocolNegotiation::Compatible(agreement) => agreement,
            UiHostProtocolNegotiation::Incompatible(_) => panic!("current protocol must negotiate"),
        };
        let first = UiHostObservationSequence::new(10_001);
        let pressed = UiHostObservationSequence::new(10_002);
        let released = UiHostObservationSequence::new(10_003);
        let key = |sequence: UiHostObservationSequence, transition: UiHostKeyTransition| {
            UiHostObservationReport::new(
                sequence,
                UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
                UiHostObservationPayload::Keyboard {
                    logical_key: UiHostKey::P,
                    physical_key: Some(UiHostKey::P),
                    modifiers: UiHostKeyboardModifiers::new(false, true, true, false, false),
                    transition,
                },
            )
        };
        let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
            protocol,
            host_session: shell.host_session_identity().as_u64(),
            presentation,
            sequences: UiHostObservationSequenceRange::new(first, released),
            loss: UiHostObservationLoss::Complete,
            reports: vec![
                UiHostObservationReport::new(
                    first,
                    UiHostObservationTimeBasis::HostMonotonicMillis(first.value()),
                    UiHostObservationPayload::WindowFocus {
                        surface: presentation.host_surface(),
                        focused: true,
                    },
                ),
                key(pressed, UiHostKeyTransition::Pressed { repeat: false }),
                key(released, UiHostKeyTransition::Released),
            ],
        })
        .expect("normalized data, never a prevalidated interaction or Intent receipt");
        self.query_lifecycle
            .as_mut()
            .unwrap()
            .cancel_next_attempt_for_test();
        probe.borrow_mut().action_dispatched = true;
        let deadline = self.intent_clock.new_attempt_deadline().unwrap();
        let ingress = shell.admit_native_intent_observations(
            worth_ui_platform_pulse::intent::platform_pulse_action_definition(),
            UiHostObservationDrain::bounded(vec![batch]).unwrap(),
            deadline,
        );
        assert!(
            matches!(
                ingress.transitions(),
                [WorthUiNativeIntentTransition::AttemptPrepared(_)]
            ),
            "real framework dispatch required: transitions={:?}, interaction_stops={:?}, duplicates={}",
            ingress
                .transitions()
                .iter()
                .map(|transition| match transition {
                    WorthUiNativeIntentTransition::AttemptPrepared(_) => "prepared",
                    WorthUiNativeIntentTransition::ConfirmationRequired(_) => "confirmation",
                    WorthUiNativeIntentTransition::Stopped(stopped) =>
                        native_intent_stop_kind(stopped.stop()),
                })
                .collect::<Vec<_>>(),
            ingress.interaction_stops().iter().map(|stop| match stop {
                worth_ui::facade::app::WorthUiNativeInteractionIngressStop::ManagedPublicationPending(_) => "managed publication pending".to_owned(),
                worth_ui::facade::app::WorthUiNativeInteractionIngressStop::ManagedObservationProgressPending(_) => "managed observation pending".to_owned(),
                worth_ui::facade::app::WorthUiNativeInteractionIngressStop::Quarantined(cause) => format!("quarantined: {cause:?}"),
                worth_ui::facade::app::WorthUiNativeInteractionIngressStop::Denied(cause) => format!("denied: {cause:?}"),
            }).collect::<Vec<_>>(),
            ingress.duplicate_batches(),
        );
        self.settle_native_intent_ingress(&mut shell, ingress);
        self.shell = Some(shell);
    }
}
