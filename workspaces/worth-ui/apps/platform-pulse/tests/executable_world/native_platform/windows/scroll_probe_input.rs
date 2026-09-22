//! Modifier-scoped native wheel input for the two-axis product journey.
use super::{
    NativePlatformContract, NativePlatformFailure, WindowsNativePlatform,
    WindowsProcessBoundNativeClientArea,
};
use crate::external_observation::{NativeClientPixelPoint, NativeInputProbeKind};
use winsafe::{co, HwKbMouse, HWND, KEYBDINPUT, MOUSEINPUT};

impl WindowsNativePlatform {
    /// Observe release/capture behavior without causing focus recovery, which
    /// could itself cancel a stale drag and counterfeit a successful release.
    pub(crate) fn move_pointer_without_focus_recovery(
        &self,
        bound: &WindowsProcessBoundNativeClientArea,
        point: NativeClientPixelPoint,
    ) -> Result<(), NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        if HWND::GetForegroundWindow().as_ref() != Some(&bound.window) {
            return Err(NativePlatformFailure::InputDelivery(
                "unpressed pointer probe requires unchanged foreground".to_owned(),
            ));
        }
        let destination = super::scroll_input_delivery::screen_point_of(observed, point)?;
        super::pointer_target::require_before_effect(&bound.window, destination)?;
        super::input_environment::actuate_and_observe_cursor(destination).map_err(|denial| {
            super::input_delivery::post_effect_failure(
                NativeInputProbeKind::Pointer,
                1,
                format!("{denial:?}"),
            )
        })?;
        self.observe_bound_client_area(bound)?;
        if HWND::GetForegroundWindow().as_ref() != Some(&bound.window) {
            return Err(super::input_delivery::post_effect_failure(
                NativeInputProbeKind::Pointer,
                1,
                "foreground changed during unpressed pointer probe",
            ));
        }
        super::pointer_target::require_after_effect(
            &bound.window,
            destination,
            NativeInputProbeKind::Pointer,
            1,
        )
    }

    pub(crate) fn drag_thumb_outside_below(
        &self,
        bound: &WindowsProcessBoundNativeClientArea,
        from: NativeClientPixelPoint,
    ) -> Result<(), NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        let (x, _) = super::scroll_input_delivery::screen_point_of(observed, from)?;
        let bottom = observed
            .bounds()
            .bottom()
            .checked_add(16)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
        super::scroll_input_delivery::deliver_captured_drag(
            &bound.window,
            observed,
            from,
            (x, bottom),
        )?;
        self.observe_bound_client_area(bound)?;
        Ok(())
    }

    pub(crate) fn deliver_shift_wheel_notch(
        &self,
        bound: &WindowsProcessBoundNativeClientArea,
        point: NativeClientPixelPoint,
    ) -> Result<(), NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        super::scroll_input_delivery::prepare_wheel_target(&bound.window, observed, point)?;
        let events = [
            HwKbMouse::Kb(KEYBDINPUT {
                wVk: co::VK::SHIFT,
                ..Default::default()
            }),
            HwKbMouse::Mouse(MOUSEINPUT {
                mouseData: (-120_i32) as u32,
                dwFlags: co::MOUSEEVENTF::WHEEL,
                ..Default::default()
            }),
            shift_up(),
        ];
        let sent = winsafe::SendInput(&events)
            .map_err(|error| NativePlatformFailure::InputDelivery(error.to_string()));
        if matches!(sent, Ok(3)) {
            return Ok(());
        }
        // A partial batch may have pressed Shift; always release on failure.
        let release = winsafe::SendInput(&[shift_up()]);
        Err(NativePlatformFailure::InputDelivery(format!(
            "Shift+wheel batch incomplete: {sent:?}; modifier cleanup: {release:?}"
        )))
    }
}

fn shift_up() -> HwKbMouse {
    HwKbMouse::Kb(KEYBDINPUT {
        wVk: co::VK::SHIFT,
        dwFlags: co::KEYEVENTF::KEYUP,
        ..Default::default()
    })
}
