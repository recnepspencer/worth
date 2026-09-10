use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

use uiautomation::patterns::UIWindowPattern;
use uiautomation::types::Handle;
use uiautomation::UIAutomation;
use winsafe::{self as win, co, HwndPlace, HWND, POINT, SIZE};
use xcap::Window;

use crate::external_observation::{
    NativeClientPixelCapture, NativeClientPixelPoint, NativeInputDeliveryObservation,
    NativeInputProbeKind, NativeWindowIdentity, NativeWindowVisibilityTransitionObservation,
    NormalNativeCloseRequestObservation, ProcessBoundNativeClientAreaObservation,
};

use super::contract::sealed::Sealed;
use super::{NativePlatformContract, NativePlatformFailure};

mod capture_consistency;
mod capture_region;
mod client_capture;
mod environment;
mod gdi_capture;
mod input_delivery;
#[cfg(test)]
mod input_delivery_tests;
mod input_environment;
mod pointer_target;
mod pointer_visual_settlement;
mod process_windows;
mod window_state;

pub(super) use input_environment::WindowsInputEnvironmentDenial;

use capture_consistency::{require_matching_capture_sources, require_matching_composited_sources};
use capture_region::{
    capture_bound_client_area, client_bounds, monitor_capture_region, monitor_for_client,
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WindowsNativePlatform {
    _private: (),
}

pub(crate) struct WindowsProcessBoundNativeClientArea {
    window: HWND,
    capture_window: Window,
    observation: ProcessBoundNativeClientAreaObservation,
}

struct WindowsCaptureExposure<'bound> {
    bound: &'bound WindowsProcessBoundNativeClientArea,
}

impl Drop for WindowsCaptureExposure<'_> {
    fn drop(&mut self) {
        let restoration = self.bound.window.SetWindowPos(
            HwndPlace::Place(co::HWND_PLACE::NOTOPMOST),
            POINT::default(),
            SIZE::default(),
            co::SWP::NOMOVE | co::SWP::NOSIZE | co::SWP::NOACTIVATE,
        );
        if let Err(error) = restoration {
            assert!(
                !self.bound.window.IsWindow(),
                "capture exposure must restore ordinary z-order for a live window: {error}"
            );
        }
    }
}

impl WindowsNativePlatform {
    pub(crate) fn certified() -> Result<Self, NativePlatformFailure> {
        if std::env::consts::ARCH != "x86_64" {
            return Err(NativePlatformFailure::EnvironmentQualification(
                std::env::consts::ARCH.to_owned(),
            ));
        }
        static WINDOWS_VERSION: OnceLock<Result<(), String>> = OnceLock::new();
        if let Err(version) = WINDOWS_VERSION.get_or_init(qualify_windows_version) {
            return Err(NativePlatformFailure::EnvironmentQualification(
                version.clone(),
            ));
        }
        static DPI_AWARENESS: OnceLock<Result<(), String>> = OnceLock::new();
        match DPI_AWARENESS
            .get_or_init(|| win::SetProcessDPIAware().map_err(|error| error.to_string()))
        {
            Ok(()) => Ok(Self { _private: () }),
            Err(error) => Err(NativePlatformFailure::DpiAwareness(error.clone())),
        }
    }

    fn expose_bound_client_area<'bound>(
        &self,
        bound: &'bound WindowsProcessBoundNativeClientArea,
    ) -> Result<WindowsCaptureExposure<'bound>, NativePlatformFailure> {
        self.observe_bound_client_area(bound)?;
        bound
            .window
            .SetWindowPos(
                HwndPlace::Place(co::HWND_PLACE::TOPMOST),
                POINT::default(),
                SIZE::default(),
                co::SWP::NOMOVE | co::SWP::NOSIZE | co::SWP::NOACTIVATE | co::SWP::SHOWWINDOW,
            )
            .map_err(|error| NativePlatformFailure::ClientExposure(error.to_string()))?;
        win::DwmFlush()
            .map_err(|error| NativePlatformFailure::ClientExposure(error.to_string()))?;
        self.observe_bound_client_area(bound)?;
        Ok(WindowsCaptureExposure { bound })
    }

    fn capture_exposed_client_area(
        exposure: WindowsCaptureExposure<'_>,
    ) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
        let bound = exposure.bound;
        let client = bound.observation.bounds();
        let monitor = monitor_for_client(client)?;
        let region = monitor_capture_region(&monitor, client)?;
        let monitor_capture =
            capture_bound_client_area(&monitor, region, bound.observation.process_id())?;
        let window_capture = client_capture::capture_client_area(
            &bound.capture_window,
            client,
            bound.observation.process_id(),
        )?;
        require_matching_capture_sources(&monitor_capture, &window_capture)?;
        let gdi_capture = gdi_capture::capture_client_area(client, bound.observation.process_id())?;
        require_matching_composited_sources(&monitor_capture, &gdi_capture)?;
        Ok(monitor_capture)
    }
}

fn qualify_windows_version() -> Result<(), String> {
    let output = std::process::Command::new("cmd")
        .args(["/c", "ver"])
        .output()
        .map_err(|error| error.to_string())?;
    let version = String::from_utf8(output.stdout)
        .map_err(|error| error.to_string())?
        .trim()
        .to_owned();
    if output.status.success() && environment::qualified_windows_11_version(&version) {
        Ok(())
    } else {
        Err(version)
    }
}

impl Sealed for WindowsNativePlatform {}

#[test]
fn independent_window_capture_rejects_monitor_pixel_substitution() {
    capture_consistency::independent_window_capture_rejects_monitor_pixel_substitution();
}

impl NativePlatformContract for WindowsNativePlatform {
    type BoundClientArea = WindowsProcessBoundNativeClientArea;

    fn bind_process_client_area(
        &self,
        process_id: u32,
        deadline: Instant,
    ) -> Result<Self::BoundClientArea, NativePlatformFailure> {
        let mut lookup_count = 0_u32;
        let mut prior_candidate = None;
        let mut stable_observations = 0_u8;
        loop {
            lookup_count = lookup_count.saturating_add(1);
            let mut candidates = process_windows::enumerate(process_id)?;
            match candidates.len() {
                0 if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(20));
                }
                0 => return Err(NativePlatformFailure::WindowLookupDeadline),
                1 => {
                    let candidate = candidates.pop().expect("one process window");
                    let identity = candidate.window.ptr() as usize;
                    let posture = (identity, candidate.bounds);
                    if prior_candidate == Some(posture) {
                        stable_observations = stable_observations.saturating_add(1);
                    } else {
                        prior_candidate = Some(posture);
                        stable_observations = 1;
                    }
                    if stable_observations < 3 && Instant::now() < deadline {
                        thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    let window_identity = NativeWindowIdentity::from_native_value(identity)
                        .expect("enumerated HWND is non-null");
                    let capture_monitor = monitor_for_client(candidate.bounds)?;
                    let _capture_region =
                        monitor_capture_region(&capture_monitor, candidate.bounds)?;
                    let capture_window =
                        client_capture::exact_window(process_id, candidate.window.ptr() as u32)?;
                    let dpi = candidate.window.GetDpiForWindow();
                    if dpi == 0 {
                        return Err(NativePlatformFailure::DpiAwareness(
                            "GetDpiForWindow returned zero".to_owned(),
                        ));
                    }
                    return Ok(WindowsProcessBoundNativeClientArea {
                        window: candidate.window,
                        capture_window,
                        observation: ProcessBoundNativeClientAreaObservation::new(
                            process_id,
                            window_identity,
                            candidate.bounds,
                            dpi,
                            lookup_count,
                        ),
                    });
                }
                count => {
                    return Err(NativePlatformFailure::AmbiguousProcessWindows(count));
                }
            }
        }
    }

    fn observe_bound_client_area(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure> {
        if !bound.window.IsWindow() {
            return Err(NativePlatformFailure::BoundWindowMissing);
        }
        let (_, owner_process_id) = bound.window.GetWindowThreadProcessId();
        if owner_process_id != bound.observation.process_id() {
            return Err(NativePlatformFailure::BoundWindowOwnerChanged);
        }
        if bound.capture_window.pid().ok() != Some(owner_process_id)
            || bound.capture_window.id().ok() != Some(bound.window.ptr() as u32)
        {
            return Err(NativePlatformFailure::BoundWindowOwnerChanged);
        }
        if client_bounds(&bound.window)? != bound.observation.bounds() {
            return Err(NativePlatformFailure::BoundClientAreaChanged);
        }
        if bound.window.GetDpiForWindow() != bound.observation.dpi() {
            return Err(NativePlatformFailure::BoundWindowDpiChanged);
        }
        Ok(bound.observation)
    }

    fn capture_client_area(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
        let exposure = self.expose_bound_client_area(bound)?;
        Self::capture_exposed_client_area(exposure)
    }

    fn resize_bound_client_area(
        &self,
        bound: &mut Self::BoundClientArea,
        client_physical_size: [u32; 2],
        deadline: Instant,
    ) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure> {
        self.observe_bound_client_area(bound)?;
        window_state::resize(bound, client_physical_size, deadline)
    }

    fn minimize_and_restore_bound_client_area(
        &self,
        bound: &mut Self::BoundClientArea,
        deadline: Instant,
    ) -> Result<NativeWindowVisibilityTransitionObservation, NativePlatformFailure> {
        self.observe_bound_client_area(bound)?;
        window_state::minimize_and_restore(bound, deadline)
    }

    fn deliver_input_reachability_probe(
        &self,
        bound: &Self::BoundClientArea,
        kind: NativeInputProbeKind,
    ) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        input_delivery::deliver(&bound.window, observed, kind)
    }

    fn deliver_pointer_activation(
        &self,
        bound: &Self::BoundClientArea,
        point: NativeClientPixelPoint,
    ) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        input_delivery::deliver_pointer(&bound.window, observed, point)
    }

    fn deliver_keyboard_command(
        &self,
        bound: &Self::BoundClientArea,
        command: crate::external_observation::NativeKeyboardCommand,
    ) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        input_delivery::deliver_keyboard_command(&bound.window, observed, command)
    }

    fn deliver_wheel_deltas(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<(), NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        input_delivery::deliver_wheel_deltas(&bound.window, observed)
    }

    fn move_cursor(&self, screen_point: (i32, i32)) -> Result<(), NativePlatformFailure> {
        input_environment::actuate_and_observe_cursor(screen_point)
            .map_err(NativePlatformFailure::InputEnvironment)
    }

    fn request_normal_close(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<NormalNativeCloseRequestObservation, NativePlatformFailure> {
        self.observe_bound_client_area(bound)?;
        let automation = UIAutomation::new()
            .map_err(|error| NativePlatformFailure::NormalClose(error.to_string()))?;
        automation
            .element_from_handle(Handle::from(bound.window.ptr() as isize))
            .and_then(|element| element.get_pattern::<UIWindowPattern>())
            .and_then(|pattern| pattern.close())
            .map_err(|error| NativePlatformFailure::NormalClose(error.to_string()))?;
        Ok(NormalNativeCloseRequestObservation::one(
            bound.observation.process_id(),
        ))
    }

    fn verify_process_window_released(&self, process_id: u32) -> Result<(), NativePlatformFailure> {
        let windows = process_windows::enumerate(process_id)?;
        if windows.is_empty() {
            Ok(())
        } else {
            Err(NativePlatformFailure::ProcessWindowResidue(windows.len()))
        }
    }
}
