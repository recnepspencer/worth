use worth_ui_host_contract::{
    UiHostObservationCanonicalCore, UiHostObservationReport, UiHostPointerIdentity,
};

use crate::runtime::interaction::gesture::{
    UiPointerGestureOutcome, UiPointerGestureRuntimeState, UiPointerGestureStopReason,
};
use crate::runtime::interaction::UiPrimaryPointerKind;

pub(super) fn process(
    state: &mut UiPointerGestureRuntimeState,
    stop: Option<(UiHostPointerIdentity, UiPointerGestureStopReason)>,
    core: UiHostObservationCanonicalCore,
    report: &UiHostObservationReport,
    kind: Option<UiPrimaryPointerKind>,
    mounted: &crate::mounting::WorthUiMountedSessionState,
) -> Vec<UiPointerGestureOutcome> {
    if let Some((pointer_identity, reason)) = stop {
        state.stop_pointer_for_denial(pointer_identity, report.sequence(), reason)
    } else {
        state.process_report(core, report, kind, mounted)
    }
}
