use worth_ui_host_contract::{UiHostObservationPayload, UiHostObservationReport};

use super::pointer_presence::{UiPointerPresenceAdmissionDenial, UiPrimaryPointerKind};

pub(super) fn admit(
    report: &UiHostObservationReport,
) -> Result<Option<UiPrimaryPointerKind>, UiPointerPresenceAdmissionDenial> {
    let pointer = match report.payload() {
        UiHostObservationPayload::PointerMotion { pointer, .. }
        | UiHostObservationPayload::PointerButton { pointer, .. } => *pointer,
        _ => return Ok(None),
    };
    report
        .admit_pointer_device_kind()
        .map(UiPrimaryPointerKind::from_host)
        .map(Some)
        .map_err(|denial| match denial {
            worth_ui_host_contract::UiHostObservationPointerDeviceKindDenial::MissingForPointerPayload(
                family,
            ) => UiPointerPresenceAdmissionDenial::MissingDeviceKind { pointer, family },
            worth_ui_host_contract::UiHostObservationPointerDeviceKindDenial::NotPointerPayload(
                _,
            ) => unreachable!("pointer payload was admitted as non-pointer"),
        })
}
