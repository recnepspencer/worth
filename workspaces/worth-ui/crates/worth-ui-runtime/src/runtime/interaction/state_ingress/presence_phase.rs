use worth_ui_host_contract::{
    UiHostObservationCanonicalCore, UiHostObservationPayload, UiHostObservationReport,
};

use crate::runtime::interaction::{
    UiPointerPresenceAdmissionDenial, UiPointerPresenceOwner, UiPointerPresenceTargetTransition,
};

use super::pointer_admission::UiPointerAdmission;

pub(super) fn process(
    owner: &mut Option<UiPointerPresenceOwner>,
    admission: &UiPointerAdmission,
    core: UiHostObservationCanonicalCore,
    report: &UiHostObservationReport,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
) -> Result<Option<UiPointerPresenceTargetTransition>, UiPointerPresenceAdmissionDenial> {
    if admission.denied() {
        return Ok(None);
    }
    match (owner.as_mut(), admission.kind(), report.payload()) {
        (Some(owner), Some(kind), UiHostObservationPayload::PointerMotion { .. }) => {
            owner.process_pointer_report(core, report, kind, mounted, generation)
        }
        (Some(owner), Some(kind), UiHostObservationPayload::PointerButton { pointer, .. }) => {
            owner.admit_pointer_kind(*pointer, kind)?;
            Ok(None)
        }
        _ => Ok(None),
    }
}
