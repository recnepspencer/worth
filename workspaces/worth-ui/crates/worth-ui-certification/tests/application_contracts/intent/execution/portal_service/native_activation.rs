use worth_ui::facade::observation_report::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationDrain,
    UiHostObservationLoss, UiHostObservationPayload, UiHostObservationPresentationBasis,
    UiHostObservationReport, UiHostObservationSequence, UiHostObservationSequenceRange,
    UiHostObservationTimeBasis, UiHostPointerButton, UiHostPointerButtonTransition,
    UiHostPointerCaptureEpoch, UiHostPointerDeviceKind, UiHostPointerIdentity,
    UiHostProtocolContract, UiHostProtocolNegotiation, UiHostSurfacePosition,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};
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
