//! Prepare focus/pointer once; trace-time delivery preserves current OS targeting.
use super::{
    NativePlatformContract, NativePlatformFailure, WindowsNativePlatform,
    WindowsProcessBoundNativeClientArea,
};
use crate::external_observation::{
    NativeClientPixelPoint, NativeInputDeliveryTiming, NativeInputProbeKind,
};
use winsafe::{co, HwKbMouse, HWND, MOUSEINPUT};

pub(crate) struct WindowsPreparedWheelInput<'bound> {
    bound: &'bound WindowsProcessBoundNativeClientArea,
    screen_point: (i32, i32),
}

impl WindowsNativePlatform {
    pub(crate) fn prepare_wheel_input<'bound>(
        &self,
        bound: &'bound WindowsProcessBoundNativeClientArea,
        point: NativeClientPixelPoint,
    ) -> Result<WindowsPreparedWheelInput<'bound>, NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        let screen_point =
            super::scroll_input_delivery::prepare_wheel_target(&bound.window, observed, point)?;
        Ok(WindowsPreparedWheelInput {
            bound,
            screen_point,
        })
    }
}

impl WindowsPreparedWheelInput<'_> {
    pub(crate) fn press_primary(
        &self,
    ) -> Result<
        (
            super::held_pointer::WindowsHeldPointer<'_>,
            NativeInputDeliveryTiming,
        ),
        NativePlatformFailure,
    > {
        super::held_pointer::WindowsHeldPointer::press(self.bound, self.screen_point)
    }

    /// +1 scrolls down, -1 scrolls up; precisely one native wheel event.
    pub(crate) fn deliver_notch(
        &self,
        direction: i32,
    ) -> Result<NativeInputDeliveryTiming, NativePlatformFailure> {
        if !matches!(direction, -1 | 1) {
            return Err(NativePlatformFailure::InputDelivery(
                "a timed notch direction must be -1 or +1".to_owned(),
            ));
        }
        WindowsNativePlatform::default().observe_bound_client_area(self.bound)?;
        let pointer = winsafe::GetCursorPos()
            .map_err(|error| NativePlatformFailure::InputDelivery(error.to_string()))?;
        if HWND::GetForegroundWindow().as_ref() != Some(&self.bound.window)
            || (pointer.x, pointer.y) != self.screen_point
        {
            return Err(NativePlatformFailure::InputDelivery(
                "prepared wheel focus or pointer changed".to_owned(),
            ));
        }
        super::pointer_target::require_before_effect(&self.bound.window, self.screen_point)?;
        let event = HwKbMouse::Mouse(MOUSEINPUT {
            mouseData: (-direction * 120) as u32,
            dwFlags: co::MOUSEEVENTF::WHEEL,
            ..Default::default()
        });
        let before_qpc_100ns = super::graphics_capture::qpc_100ns()?;
        let delivered = winsafe::SendInput(&[event])
            .map_err(|error| NativePlatformFailure::InputDelivery(error.to_string()))?;
        // Once input may have escaped, timestamp/target failures are not a
        // pre-effect denial. The caller must reject the entire trace.
        let after_qpc_100ns = super::graphics_capture::qpc_100ns().map_err(|error| {
            super::input_delivery::post_effect_failure(
                NativeInputProbeKind::Pointer,
                delivered,
                error.to_string(),
            )
        })?;
        if delivered != 1 || after_qpc_100ns < before_qpc_100ns {
            return Err(super::input_delivery::post_effect_failure(
                NativeInputProbeKind::Pointer,
                delivered,
                "timed wheel delivery or QPC bracket invalid".to_owned(),
            ));
        }
        super::pointer_target::require_after_effect(
            &self.bound.window,
            self.screen_point,
            NativeInputProbeKind::Pointer,
            delivered,
        )?;
        Ok(NativeInputDeliveryTiming {
            before_qpc_100ns,
            after_qpc_100ns,
        })
    }
}
