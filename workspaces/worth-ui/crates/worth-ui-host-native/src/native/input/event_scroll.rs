use winit::event::{MouseScrollDelta, TouchPhase, WindowEvent};
use worth_ui_host_contract::{
    UiHostObservationPayload, UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision,
    UiHostScrollDeltaSource, UiHostScrollDeltaTargetAffinity,
};

use super::{
    pointer, UiNativeInputObservationDisposition, UiNativeInputObservationEventFamily,
    UiNativeInputObservationState, UiNativeInputObservationStop, UiNativePointerPositionWitness,
};

pub(super) fn observe(
    state: &mut UiNativeInputObservationState,
    event: &WindowEvent,
    pointer_witness: UiNativePointerPositionWitness,
) -> Option<UiNativeInputObservationDisposition> {
    let WindowEvent::MouseWheel { delta, phase, .. } = event else {
        return None;
    };
    if !state.admit_input(UiNativeInputObservationEventFamily::Scroll) {
        return Some(state.rejection_disposition());
    }
    let Some(profile) = state.profile.profile() else {
        return Some(state.terminal_disposition(UiNativeInputObservationStop::MissingEventProfile));
    };
    let (x_subpixels, y_subpixels, precision) =
        match delta {
            MouseScrollDelta::PixelDelta(delta) => {
                match pointer::logical_delta(*delta, profile.scale_factor) {
                    Ok((x, y)) => (x, y, UiHostScrollDeltaPrecision::Pixel),
                    Err(pointer::UiNativePointerCoordinateDenial::NotFinite) => {
                        return Some(state.terminal_disposition(
                            UiNativeInputObservationStop::CoordinateNotFinite,
                        ));
                    }
                    Err(pointer::UiNativePointerCoordinateDenial::OutOfRange) => {
                        return Some(state.terminal_disposition(
                            UiNativeInputObservationStop::CoordinateOutOfRange,
                        ));
                    }
                }
            }
            MouseScrollDelta::LineDelta(x_notches, y_notches) => {
                match pointer::logical_line_delta(
                    *x_notches,
                    *y_notches,
                    profile.wheel_units_per_notch(),
                ) {
                    Ok((x, y)) => (x, y, profile.wheel_precision()),
                    Err(pointer::UiNativePointerCoordinateDenial::NotFinite) => {
                        return Some(state.terminal_disposition(
                            UiNativeInputObservationStop::CoordinateNotFinite,
                        ));
                    }
                    Err(pointer::UiNativePointerCoordinateDenial::OutOfRange) => {
                        return Some(state.terminal_disposition(
                            UiNativeInputObservationStop::CoordinateOutOfRange,
                        ));
                    }
                }
            }
        };
    // Modifiers are event-time host input facts. Shift redirects only a purely
    // vertical wheel; an explicit inline delta (including diagonal precision
    // input) keeps both axes intact. Runtime still owns region axis policy.
    let (x_subpixels, y_subpixels) = if state.modifiers.shift() && x_subpixels == 0 {
        (y_subpixels, 0)
    } else {
        (x_subpixels, y_subpixels)
    };
    let Some((_, _, presentation)) = state.completed else {
        return Some(state.rejection_disposition());
    };
    let target =
        match pointer_witness {
            UiNativePointerPositionWitness::EventTime(position) => {
                match pointer::logical_position(position, profile.scale_factor) {
                    Ok(position) => {
                        UiHostScrollDeltaTargetAffinity::exact_coordinate(presentation, position)
                    }
                    Err(pointer::UiNativePointerCoordinateDenial::NotFinite) => {
                        return Some(state.terminal_disposition(
                            UiNativeInputObservationStop::CoordinateNotFinite,
                        ));
                    }
                    Err(pointer::UiNativePointerCoordinateDenial::OutOfRange) => {
                        return Some(state.terminal_disposition(
                            UiNativeInputObservationStop::CoordinateOutOfRange,
                        ));
                    }
                }
            }
            UiNativePointerPositionWitness::Unavailable => {
                UiHostScrollDeltaTargetAffinity::presented_surface_fallback(presentation)
            }
        };
    Some(state.emit_payloads([UiHostObservationPayload::ScrollDelta {
        source: UiHostScrollDeltaSource::PointerWheel,
        phase: scroll_phase(*phase),
        precision,
        target,
        x_subpixels,
        y_subpixels,
    }]))
}

const fn scroll_phase(phase: TouchPhase) -> UiHostScrollDeltaPhase {
    match phase {
        TouchPhase::Started => UiHostScrollDeltaPhase::Started,
        TouchPhase::Moved => UiHostScrollDeltaPhase::Updated,
        TouchPhase::Ended => UiHostScrollDeltaPhase::Ended,
        TouchPhase::Cancelled => UiHostScrollDeltaPhase::Cancelled,
    }
}
