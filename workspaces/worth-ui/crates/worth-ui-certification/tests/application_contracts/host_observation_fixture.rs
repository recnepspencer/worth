use worth_ui::facade::app::WorthUiActiveApplicationSession;
use worth_ui::facade::observation_report::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationLoss,
    UiHostObservationMountedBasis, UiHostObservationPayload, UiHostObservationPresentationBasis,
    UiHostObservationReport, UiHostObservationSequence, UiHostObservationSequenceRange,
    UiHostObservationTimeBasis, UiHostPointerCaptureEpoch, UiHostPointerDeviceKind,
    UiHostPointerIdentity, UiHostPressedPointerButtons, UiHostProtocolContract,
    UiHostProtocolNegotiation, UiHostSurfacePosition,
};
use worth_ui_runtime::facade::mounted::UiSurfaceBindingGeneration;

use super::mounted_application_lifecycle::published_mounted_world::PresentedObservationBasis;

#[derive(Clone, Copy)]
pub(super) struct HostObservationSource<'a> {
    session: &'a WorthUiActiveApplicationSession,
    binding: UiSurfaceBindingGeneration,
    basis: &'a PresentedObservationBasis,
}

pub(super) fn report(
    sequence: u64,
    payload: UiHostObservationPayload,
    basis: &PresentedObservationBasis,
) -> UiHostObservationReport {
    let report = UiHostObservationReport::new(
        UiHostObservationSequence::new(sequence),
        UiHostObservationTimeBasis::HostMonotonicMillis(sequence),
        payload,
    )
    .with_mounted_basis(UiHostObservationMountedBasis::new(
        basis.instance,
        basis.receipt,
    ));
    if matches!(
        report.payload(),
        UiHostObservationPayload::PointerMotion { .. }
            | UiHostObservationPayload::PointerButton { .. }
    ) {
        report
            .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
            .expect("pointer fixture reports carry an explicit host kind")
    } else {
        report
    }
}

pub(super) fn window_focus(
    basis: &PresentedObservationBasis,
    focused: bool,
) -> UiHostObservationPayload {
    UiHostObservationPayload::WindowFocus {
        surface: basis.host_surface,
        focused,
    }
}

pub(super) fn batch(
    source: HostObservationSource<'_>,
    range: (u64, u64),
    loss: UiHostObservationLoss,
    reports: Vec<UiHostObservationReport>,
) -> UiHostObservationBatch {
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol: protocol(),
        host_session: source.session.host_session_identity().as_u64(),
        presentation: UiHostObservationPresentationBasis::new(
            source.basis.host_surface,
            source.basis.frame,
            source.binding,
            source.basis.epoch,
        ),
        sequences: UiHostObservationSequenceRange::new(
            UiHostObservationSequence::new(range.0),
            UiHostObservationSequence::new(range.1),
        ),
        loss,
        reports,
    })
    .expect("authored raw batch shape is valid")
}

pub(super) fn source<'a>(
    session: &'a WorthUiActiveApplicationSession,
    binding: UiSurfaceBindingGeneration,
    basis: &'a PresentedObservationBasis,
) -> HostObservationSource<'a> {
    HostObservationSource {
        session,
        binding,
        basis,
    }
}

pub(super) fn pointer(sequence: u64, x: i64) -> UiHostObservationPayload {
    UiHostObservationPayload::PointerMotion {
        pointer: UiHostPointerIdentity::new(7),
        capture_epoch: UiHostPointerCaptureEpoch::new(3),
        pressed_buttons: UiHostPressedPointerButtons::NONE,
        position: UiHostSurfacePosition::viewport_logical(x, i64::try_from(sequence).unwrap()),
    }
}

pub(super) fn protocol() -> worth_ui::facade::observation_report::UiHostProtocolAgreement {
    match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => {
            panic!("current protocol must negotiate")
        }
    }
}
