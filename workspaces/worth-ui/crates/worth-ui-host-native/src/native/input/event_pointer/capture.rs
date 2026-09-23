use super::super::{
    UiNativeInputObservationDisposition, UiNativeInputObservationEventFamily,
    UiNativeInputObservationState, UiNativeInputObservationStop,
};

pub(super) fn observe_exit(
    state: &mut UiNativeInputObservationState,
) -> UiNativeInputObservationDisposition {
    if !state.admit_input(UiNativeInputObservationEventFamily::Pointer) {
        return state.rejection_disposition();
    }
    // Leaving the client is not capture loss during a held drag. Preserve its
    // epoch so outside motion and release reach the same runtime capture.
    // Focus loss and session teardown still cancel held capture explicitly.
    if !state.pointer.has_pressed_buttons() && state.pointer.end_capture().is_err() {
        state.record_terminal_stop(UiNativeInputObservationStop::PointerCaptureEpochExhausted);
        return UiNativeInputObservationDisposition::Stopped;
    }
    UiNativeInputObservationDisposition::Ignored
}
