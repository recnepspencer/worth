//! The Linux X11 observer (milestone-3.10.3 §D5 successor lane): the party
//! that binds, captures, actuates and closes the product's window through
//! the X protocol itself, on a bare Xvfb display with no window manager. It
//! plays the ICCCM manager's part where one is needed (close request) and
//! declares, on the observation and in the profile record, where its
//! mechanism differs from the Windows lane's (visibility by occlusion,
//! capture agreement rather than capture independence).
use std::fmt;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

use x11rb::protocol::xproto::Window;

use crate::external_observation::{
    NativeClientPixelCapture, NativeClientPixelPoint, NativeInputDeliveryObservation,
    NativeInputProbeKind, NativeWindowIdentity, NativeWindowVisibilityTransitionMechanism,
    NativeWindowVisibilityTransitionObservation, NormalNativeCloseRequestObservation,
    ProcessBoundNativeClientAreaObservation,
};

use super::contract::sealed::Sealed;
use super::{CreationSurfaceSuccession, NativePlatformContract, NativePlatformFailure};

mod capture_agreement;
mod client_capture;
mod connection;
mod environment;
mod input_delivery;
mod input_environment;
mod keyboard_map;
mod normal_close;
mod pixel_layout;
mod pointer_target;
mod process_windows;
mod scale;
mod window_state;

pub(crate) use input_environment::LinuxX11InputEnvironmentDenial;

use connection::{X11Observation, X11QualificationDenial};

/// `Copy`: the value is a proof that the process-wide display was qualified,
/// carrying a reference to it.
#[derive(Clone, Copy)]
pub(crate) struct LinuxX11NativePlatform {
    x11: &'static X11Observation,
}

pub(crate) struct LinuxX11ProcessBoundNativeClientArea {
    window: Window,
    observation: ProcessBoundNativeClientAreaObservation,
}

impl fmt::Debug for LinuxX11NativePlatform {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LinuxX11NativePlatform")
            .field("root", &self.x11.root())
            .finish()
    }
}

impl LinuxX11NativePlatform {
    /// The visibility transition this observer actuates; the profile record's
    /// `client_visibility_transition_observation` names the same mechanism.
    pub(crate) const VISIBILITY_TRANSITION_MECHANISM: NativeWindowVisibilityTransitionMechanism =
        NativeWindowVisibilityTransitionMechanism::FullOcclusion;

    pub(crate) fn certified() -> Result<Self, NativePlatformFailure> {
        if std::env::consts::ARCH != "x86_64" {
            return Err(NativePlatformFailure::EnvironmentQualification(
                std::env::consts::ARCH.to_owned(),
            ));
        }
        static OBSERVATION: OnceLock<Result<X11Observation, X11QualificationDenial>> =
            OnceLock::new();
        match OBSERVATION.get_or_init(X11Observation::connect_qualified) {
            Ok(x11) => Ok(Self { x11 }),
            Err(denial) => Err(denial.clone().into()),
        }
    }
}

impl Sealed for LinuxX11NativePlatform {}

impl NativePlatformContract for LinuxX11NativePlatform {
    type BoundClientArea = LinuxX11ProcessBoundNativeClientArea;

    const CREATION_SURFACE_SUCCESSION: CreationSurfaceSuccession = CreationSurfaceSuccession::None;

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
            let mut candidates = process_windows::enumerate(self.x11, process_id)?;
            match candidates.len() {
                0 if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
                0 => return Err(NativePlatformFailure::WindowLookupDeadline),
                1 => {
                    let candidate = candidates.pop().expect("one process window");
                    let posture = (candidate.window, candidate.bounds);
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
                    let identity =
                        NativeWindowIdentity::from_native_value(candidate.window as usize)
                            .ok_or_else(|| {
                                NativePlatformFailure::WindowEnumeration(
                                    "the server answered a null window id".to_owned(),
                                )
                            })?;
                    client_capture::require_within_screen(self.x11, candidate.bounds)?;
                    return Ok(LinuxX11ProcessBoundNativeClientArea {
                        window: candidate.window,
                        observation: ProcessBoundNativeClientAreaObservation::new(
                            process_id,
                            identity,
                            candidate.bounds,
                            self.x11.dpi(),
                            lookup_count,
                        ),
                    });
                }
                count => return Err(NativePlatformFailure::AmbiguousProcessWindows(count)),
            }
        }
    }

    fn observe_bound_client_area(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure> {
        let current = window_state::current_observation(self.x11, bound)?;
        if current.bounds() != bound.observation.bounds() {
            return Err(NativePlatformFailure::BoundClientAreaChanged);
        }
        Ok(bound.observation)
    }

    fn capture_client_area(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        window_state::expose(self.x11, bound.window)?;
        let observed_after_exposure = self.observe_bound_client_area(bound)?;
        debug_assert_eq!(observed, observed_after_exposure);
        let bounds = observed.bounds();
        client_capture::require_within_screen(self.x11, bounds)?;
        let window =
            client_capture::capture_window(self.x11, bound.window, bounds, observed.process_id())?;
        let root = client_capture::capture_root_at(self.x11, bounds, observed.process_id())?;
        capture_agreement::require_request_agreement(&window, &root)?;
        Ok(window)
    }

    fn resize_bound_client_area(
        &self,
        bound: &mut Self::BoundClientArea,
        client_physical_size: [u32; 2],
        deadline: Instant,
    ) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure> {
        self.observe_bound_client_area(bound)?;
        window_state::resize(self.x11, bound, client_physical_size, deadline)
    }

    fn minimize_and_restore_bound_client_area(
        &self,
        bound: &mut Self::BoundClientArea,
        deadline: Instant,
    ) -> Result<NativeWindowVisibilityTransitionObservation, NativePlatformFailure> {
        self.observe_bound_client_area(bound)?;
        window_state::occlude_and_uncover(self.x11, bound, deadline)
    }

    fn deliver_input_reachability_probe(
        &self,
        bound: &Self::BoundClientArea,
        kind: NativeInputProbeKind,
    ) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        input_delivery::deliver(self.x11, bound.window, observed, kind)
    }

    fn deliver_pointer_activation(
        &self,
        bound: &Self::BoundClientArea,
        point: NativeClientPixelPoint,
    ) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        input_delivery::deliver_pointer(self.x11, bound.window, observed, point)
    }

    fn deliver_keyboard_command(
        &self,
        bound: &Self::BoundClientArea,
        command: crate::external_observation::NativeKeyboardCommand,
    ) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        input_delivery::deliver_keyboard_command(self.x11, bound.window, observed, command)
    }

    fn deliver_wheel_deltas(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<(), NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        input_delivery::deliver_wheel_deltas(self.x11, bound.window, observed)
    }

    fn move_cursor(&self, screen_point: (i32, i32)) -> Result<(), NativePlatformFailure> {
        input_environment::actuate_and_observe_cursor(self.x11, screen_point)
            .map_err(NativePlatformFailure::InputEnvironment)
    }

    fn request_normal_close(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<NormalNativeCloseRequestObservation, NativePlatformFailure> {
        let observed = self.observe_bound_client_area(bound)?;
        normal_close::request(self.x11, bound.window, observed.process_id())
    }

    fn verify_process_window_released(&self, process_id: u32) -> Result<(), NativePlatformFailure> {
        let windows = process_windows::enumerate(self.x11, process_id)?;
        if windows.is_empty() {
            Ok(())
        } else {
            Err(NativePlatformFailure::ProcessWindowResidue(windows.len()))
        }
    }
}
