//! Held real pointer input without focus recovery after wheel motion begins.
use super::{
    NativePlatformContract, NativePlatformFailure, WindowsNativePlatform,
    WindowsProcessBoundNativeClientArea,
};
use crate::external_observation::{
    NativeClientPixelPoint, NativeInputDeliveryTiming, NativeInputProbeKind,
};
use winsafe::{co, HwKbMouse, HWND, MOUSEINPUT};

pub(crate) struct WindowsHeldPointer<'bound> {
    bound: &'bound WindowsProcessBoundNativeClientArea,
    held: bool,
}

impl<'bound> WindowsHeldPointer<'bound> {
    pub(super) fn press(
        bound: &'bound WindowsProcessBoundNativeClientArea,
        screen: (i32, i32),
    ) -> Result<(Self, NativeInputDeliveryTiming), NativePlatformFailure> {
        WindowsNativePlatform::default().observe_bound_client_area(bound)?;
        require_foreground(bound)?;
        let pointer = winsafe::GetCursorPos().map_err(failure)?;
        if (pointer.x, pointer.y) != screen {
            return Err(failure("prepared thumb pointer changed"));
        }
        super::pointer_target::require_before_effect(&bound.window, screen)?;
        let before_qpc_100ns = super::graphics_capture::qpc_100ns()?;
        let delivered =
            winsafe::SendInput(&[button(co::MOUSEEVENTF::LEFTDOWN)]).map_err(failure)?;
        let held = Self { bound, held: true };
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
                "thumb press delivery/timestamp invalid",
            ));
        }
        Ok((
            held,
            NativeInputDeliveryTiming {
                before_qpc_100ns,
                after_qpc_100ns,
            },
        ))
    }

    pub(crate) fn move_to(
        &self,
        point: NativeClientPixelPoint,
    ) -> Result<(), NativePlatformFailure> {
        let observed = WindowsNativePlatform::default().observe_bound_client_area(self.bound)?;
        require_foreground(self.bound)?;
        let screen = super::scroll_input_delivery::screen_point_of(observed, point)?;
        super::pointer_target::require_before_effect(&self.bound.window, screen)?;
        super::input_environment::actuate_and_observe_cursor(screen).map_err(|error| {
            super::input_delivery::post_effect_failure(
                NativeInputProbeKind::Pointer,
                1,
                format!("{error:?}"),
            )
        })?;
        super::pointer_target::require_after_effect(
            &self.bound.window,
            screen,
            NativeInputProbeKind::Pointer,
            1,
        )
    }

    pub(crate) fn release(mut self) -> Result<(), NativePlatformFailure> {
        let sent = winsafe::SendInput(&[button(co::MOUSEEVENTF::LEFTUP)]).map_err(failure)?;
        if sent != 1 {
            return Err(super::input_delivery::post_effect_failure(
                NativeInputProbeKind::Pointer,
                sent,
                "held thumb release incomplete",
            ));
        }
        self.held = false;
        Ok(())
    }
}

impl Drop for WindowsHeldPointer<'_> {
    fn drop(&mut self) {
        if self.held {
            if !matches!(
                winsafe::SendInput(&[button(co::MOUSEEVENTF::LEFTUP)]),
                Ok(1)
            ) {
                eprintln!("failed to release native held-thumb probe during cleanup");
            }
        }
    }
}

fn button(flags: co::MOUSEEVENTF) -> HwKbMouse {
    HwKbMouse::Mouse(MOUSEINPUT {
        dwFlags: flags,
        ..Default::default()
    })
}

fn require_foreground(
    bound: &WindowsProcessBoundNativeClientArea,
) -> Result<(), NativePlatformFailure> {
    if HWND::GetForegroundWindow().as_ref() != Some(&bound.window) {
        return Err(failure("held thumb probe lost foreground"));
    }
    Ok(())
}

fn failure(error: impl std::fmt::Display) -> NativePlatformFailure {
    NativePlatformFailure::InputDelivery(error.to_string())
}
