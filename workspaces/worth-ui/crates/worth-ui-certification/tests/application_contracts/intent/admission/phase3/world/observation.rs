use worth_ui::facade::interaction::UiHostInteractionIngressOutcome;
use worth_ui::facade::observation_report::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationLoss,
    UiHostObservationPayload, UiHostObservationReport, UiHostObservationSequence,
    UiHostObservationSequenceRange, UiHostObservationTimeBasis, UiHostPointerButton,
    UiHostPointerButtonTransition, UiHostPointerCaptureEpoch, UiHostPointerDeviceKind,
    UiHostPointerIdentity, UiHostProtocolContract, UiHostProtocolNegotiation,
    UiHostSurfacePosition, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

use super::AdmissionWorld;

impl AdmissionWorld {
    pub(super) fn observe(
        &mut self,
        target: usize,
        payload: UiHostObservationPayload,
    ) -> UiHostInteractionIngressOutcome {
        let presentation = self.targets[target].presentation;
        let sequence = UiHostObservationSequence::new(self.next_sequence);
        self.next_sequence += 1;
        let report = UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            payload,
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .expect("pointer reports carry an explicit device kind");
        let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
            protocol: protocol(),
            host_session: self.session.host_session_identity().as_u64(),
            presentation,
            sequences: UiHostObservationSequenceRange::new(sequence, sequence),
            loss: UiHostObservationLoss::Complete,
            reports: vec![report],
        })
        .expect("admission world emits a structurally valid host batch");
        self.session.admit_host_interaction_batch(batch)
    }
}

pub(super) fn pointer_button(
    pointer: u64,
    transition: UiHostPointerButtonTransition,
    target_point: [i64; 2],
) -> UiHostObservationPayload {
    UiHostObservationPayload::PointerButton {
        pointer: UiHostPointerIdentity::new(pointer),
        capture_epoch: UiHostPointerCaptureEpoch::new(1),
        button: UiHostPointerButton::Primary,
        transition,
        position: position(target_point),
    }
}

pub(super) fn position(point: [i64; 2]) -> UiHostSurfacePosition {
    UiHostSurfacePosition::viewport_logical(
        point[0] * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
        point[1] * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
    )
}

fn protocol() -> worth_ui::facade::observation_report::UiHostProtocolAgreement {
    match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    }
}
