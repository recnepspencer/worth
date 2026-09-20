use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use worth_ui_host_contract::{
    UiHostObservationPayload, UiHostPointerButton, UiHostPointerButtonTransition,
    UiHostPointerCaptureEpoch, UiHostPointerIdentity, UiHostPressedPointerButtons,
    UiHostSurfacePosition, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

const CANONICAL_I64_EXCLUSIVE_MAX: f64 = 9_223_372_036_854_775_808.0;
#[derive(Clone, Copy, Debug)]
pub(crate) enum UiNativePointerPositionWitness {
    EventTime(PhysicalPosition<f64>),
    Unavailable,
}

#[derive(Debug)]
pub(crate) enum UiNativePointerCoordinateDenial {
    NotFinite,
    OutOfRange,
}

pub(crate) struct UiNativePointerState {
    capture_epoch: u64,
    pressed: [bool; 5],
}

impl UiNativePointerState {
    pub(crate) const fn new() -> Self {
        Self {
            capture_epoch: 1,
            pressed: [false; 5],
        }
    }

    pub(crate) fn set_pressed(&mut self, button: UiHostPointerButton, pressed: bool) {
        self.pressed[button_index(button)] = pressed;
    }

    pub(crate) fn motion(&self, position: UiHostSurfacePosition) -> UiHostObservationPayload {
        UiHostObservationPayload::PointerMotion {
            pointer: UiHostPointerIdentity::new(1),
            capture_epoch: UiHostPointerCaptureEpoch::new(self.capture_epoch),
            pressed_buttons: self.pressed_buttons(),
            position,
        }
    }

    pub(crate) fn button(
        &self,
        button: UiHostPointerButton,
        transition: UiHostPointerButtonTransition,
        position: UiHostSurfacePosition,
    ) -> UiHostObservationPayload {
        UiHostObservationPayload::PointerButton {
            pointer: UiHostPointerIdentity::new(1),
            capture_epoch: UiHostPointerCaptureEpoch::new(self.capture_epoch),
            button,
            transition,
            position,
        }
    }

    pub(crate) fn end_capture(&mut self) -> Result<(), ()> {
        self.capture_epoch = self.capture_epoch.checked_add(1).ok_or(())?;
        self.pressed = [false; 5];
        Ok(())
    }

    #[cfg(test)]
    pub(super) const fn capture_epoch(&self) -> u64 {
        self.capture_epoch
    }

    #[cfg(test)]
    pub(super) fn set_capture_epoch_for_test(&mut self, value: u64) {
        self.capture_epoch = value;
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }

    fn pressed_buttons(&self) -> UiHostPressedPointerButtons {
        UiHostPressedPointerButtons::from_buttons(
            [
                UiHostPointerButton::Primary,
                UiHostPointerButton::Secondary,
                UiHostPointerButton::Middle,
                UiHostPointerButton::Extra1,
                UiHostPointerButton::Extra2,
            ]
            .into_iter()
            .enumerate()
            .filter_map(|(index, button)| self.pressed[index].then_some(button)),
        )
    }
}

pub(crate) fn button(button: MouseButton) -> Option<UiHostPointerButton> {
    match button {
        MouseButton::Left => Some(UiHostPointerButton::Primary),
        MouseButton::Right => Some(UiHostPointerButton::Secondary),
        MouseButton::Middle => Some(UiHostPointerButton::Middle),
        MouseButton::Back => Some(UiHostPointerButton::Extra1),
        MouseButton::Forward => Some(UiHostPointerButton::Extra2),
        MouseButton::Other(_) => None,
    }
}

pub(crate) fn logical_position(
    position: PhysicalPosition<f64>,
    scale_factor: f64,
) -> Result<UiHostSurfacePosition, UiNativePointerCoordinateDenial> {
    Ok(UiHostSurfacePosition::viewport_logical(
        canonical_subpixels(position.x / scale_factor)?,
        canonical_subpixels(position.y / scale_factor)?,
    ))
}

pub(crate) fn logical_delta(
    delta: PhysicalPosition<f64>,
    scale_factor: f64,
) -> Result<(i64, i64), UiNativePointerCoordinateDenial> {
    Ok((
        canonical_subpixels(delta.x / scale_factor)?,
        canonical_subpixels(delta.y / scale_factor)?,
    ))
}

/// Turn a wheel delta counted in notches into a delta counted in lines.
///
/// The host never converts a notch into content distance. It multiplies the
/// notches the platform turned by the line count the platform states per notch,
/// and the result is carried as lines at the canonical subpixel scale.
pub(crate) fn logical_line_delta(
    x_notches: f32,
    y_notches: f32,
    lines_per_notch: f64,
) -> Result<(i64, i64), UiNativePointerCoordinateDenial> {
    Ok((
        canonical_subpixels(f64::from(x_notches) * lines_per_notch)?,
        canonical_subpixels(f64::from(y_notches) * lines_per_notch)?,
    ))
}

fn canonical_subpixels(value: f64) -> Result<i64, UiNativePointerCoordinateDenial> {
    if !value.is_finite() {
        return Err(UiNativePointerCoordinateDenial::NotFinite);
    }
    let scaled = (value * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64).round();
    if !(-CANONICAL_I64_EXCLUSIVE_MAX..CANONICAL_I64_EXCLUSIVE_MAX).contains(&scaled) {
        return Err(UiNativePointerCoordinateDenial::OutOfRange);
    }
    Ok(scaled as i64)
}

fn button_index(button: UiHostPointerButton) -> usize {
    match button {
        UiHostPointerButton::Primary => 0,
        UiHostPointerButton::Secondary => 1,
        UiHostPointerButton::Middle => 2,
        UiHostPointerButton::Extra1 => 3,
        UiHostPointerButton::Extra2 => 4,
    }
}
