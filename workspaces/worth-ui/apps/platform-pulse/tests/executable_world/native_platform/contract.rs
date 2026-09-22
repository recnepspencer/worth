#[cfg(target_os = "windows")]
use std::fmt;
#[cfg(target_os = "windows")]
use std::time::Instant;

#[cfg(target_os = "windows")]
use super::windows::WindowsInputEnvironmentDenial;
#[cfg(target_os = "windows")]
use crate::external_observation::{
    NativeClientPixelCapture, NativeClientPixelPoint, NativeInputDeliveryObservation,
    NativeInputProbeKind, NativeWindowVisibilityTransitionObservation,
    NormalNativeCloseRequestObservation, ProcessBoundNativeClientAreaObservation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativePlatformPosture {
    CertifiedExecutable,
    #[cfg(not(target_os = "windows"))]
    CompileOnly,
}

#[derive(Debug)]
#[cfg(target_os = "windows")]
pub(crate) enum NativePlatformFailure {
    DpiAwareness(String),
    EnvironmentQualification(String),
    WindowEnumeration(String),
    WindowLookupDeadline,
    AmbiguousProcessWindows(usize),
    ClientCapture(String),
    ClientPixelDeadline(&'static str),
    ClientExposure(String),
    WindowActuation(String),
    WindowStateDeadline(&'static str),
    InvalidCaptureWindowBounds,
    BoundWindowMissing,
    BoundWindowOwnerChanged,
    BoundClientAreaChanged,
    BoundWindowDpiChanged,
    ClientOutsideCaptureMonitor,
    InvalidClientCapture {
        image_width: u32,
        image_height: u32,
        outer: crate::external_observation::NativeClientAreaBounds,
        client: crate::external_observation::NativeClientAreaBounds,
    },
    NormalClose(String),
    InputEnvironment(WindowsInputEnvironmentDenial),
    InputDelivery(String),
    InputDeliveryIndeterminate {
        kind: NativeInputProbeKind,
        delivered_event_count: u32,
        detail: String,
    },
    ProcessWindowResidue(usize),
}

#[cfg(target_os = "windows")]
impl fmt::Display for NativePlatformFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DpiAwareness(error) => {
                write!(formatter, "establish process DPI awareness: {error}")
            }
            Self::EnvironmentQualification(error) => {
                write!(formatter, "qualify native environment: {error}")
            }
            Self::WindowEnumeration(error) => {
                write!(formatter, "enumerate process windows: {error}")
            }
            Self::WindowLookupDeadline => {
                formatter.write_str("process-bound native window lookup deadline elapsed")
            }
            Self::AmbiguousProcessWindows(count) => {
                write!(formatter, "found {count} visible process windows")
            }
            Self::ClientCapture(error) => write!(formatter, "capture native client area: {error}"),
            Self::ClientPixelDeadline(posture) => {
                write!(formatter, "native client pixels did not reach {posture} before deadline")
            }
            Self::ClientExposure(error) => {
                write!(formatter, "expose native client area for capture: {error}")
            }
            Self::WindowActuation(error) => write!(formatter, "actuate native window: {error}"),
            Self::WindowStateDeadline(state) => {
                write!(formatter, "native window did not reach {state} before its deadline")
            }
            Self::InvalidCaptureWindowBounds => {
                formatter.write_str("native capture window reported invalid bounds")
            }
            Self::BoundWindowMissing => {
                formatter.write_str("the process-bound native window no longer exists")
            }
            Self::BoundWindowOwnerChanged => {
                formatter.write_str("the bound native window no longer belongs to the child")
            }
            Self::BoundClientAreaChanged => {
                formatter.write_str("the bound native client area changed after observation")
            }
            Self::BoundWindowDpiChanged => {
                formatter.write_str("the bound native window DPI changed after observation")
            }
            Self::ClientOutsideCaptureMonitor => {
                formatter.write_str("the native client area is not contained by one monitor")
            }
            Self::InvalidClientCapture {
                image_width,
                image_height,
                outer,
                client,
            } => write!(
                formatter,
                "native client crop is invalid: image={image_width}x{image_height}, outer={outer:?}, client={client:?}"
            ),
            Self::NormalClose(error) => {
                write!(formatter, "request normal native-window close: {error}")
            }
            Self::InputEnvironment(denial) => {
                write!(formatter, "native input environment denied: {denial}")
            }
            Self::InputDelivery(error) => write!(formatter, "deliver native input: {error}"),
            Self::InputDeliveryIndeterminate {
                kind,
                delivered_event_count,
                detail,
            } => write!(
                formatter,
                "native {kind:?} input became indeterminate after {delivered_event_count} delivered event(s): {detail}"
            ),
            Self::ProcessWindowResidue(count) => {
                write!(formatter, "{count} process window(s) remained after exit")
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub(crate) trait NativePlatformContract: sealed::Sealed {
    type BoundClientArea;

    fn bind_process_client_area(
        &self,
        process_id: u32,
        deadline: Instant,
    ) -> Result<Self::BoundClientArea, NativePlatformFailure>;

    fn observe_bound_client_area(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure>;

    fn capture_client_area(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<NativeClientPixelCapture, NativePlatformFailure>;

    /// The bound client area held where its pixels can be read.
    ///
    /// Dropping it returns the window to where the desktop had it.
    type ExposedClientArea<'bound>
    where
        Self: 'bound;

    /// Raise the bound client area and wait for the desktop to compose it.
    ///
    /// Raising a window and waiting for a composition costs tens of
    /// milliseconds -- the same order as the intervals a motion criterion
    /// measures. It is a precondition of reading the window's pixels at all,
    /// not part of any interval measured through them, so a timing probe
    /// exposes the area once and samples inside that exposure.
    fn expose_client_area<'bound>(
        &self,
        bound: &'bound Self::BoundClientArea,
    ) -> Result<Self::ExposedClientArea<'bound>, NativePlatformFailure>;

    fn resize_bound_client_area(
        &self,
        bound: &mut Self::BoundClientArea,
        client_physical_size: [u32; 2],
        deadline: Instant,
    ) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure>;

    fn minimize_and_restore_bound_client_area(
        &self,
        bound: &mut Self::BoundClientArea,
        deadline: Instant,
    ) -> Result<NativeWindowVisibilityTransitionObservation, NativePlatformFailure>;

    fn deliver_input_reachability_probe(
        &self,
        bound: &Self::BoundClientArea,
        kind: NativeInputProbeKind,
    ) -> Result<NativeInputDeliveryObservation, NativePlatformFailure>;

    fn deliver_pointer_activation(
        &self,
        bound: &Self::BoundClientArea,
        point: NativeClientPixelPoint,
    ) -> Result<NativeInputDeliveryObservation, NativePlatformFailure>;

    fn deliver_keyboard_command(
        &self,
        bound: &Self::BoundClientArea,
        command: crate::external_observation::NativeKeyboardCommand,
    ) -> Result<NativeInputDeliveryObservation, NativePlatformFailure>;

    fn deliver_wheel_deltas(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<(), NativePlatformFailure>;

    fn deliver_wheel_notches(
        &self,
        bound: &Self::BoundClientArea,
        point: NativeClientPixelPoint,
        notches: i32,
    ) -> Result<Instant, NativePlatformFailure>;

    fn deliver_pointer_drag(
        &self,
        bound: &Self::BoundClientArea,
        from: NativeClientPixelPoint,
        to: NativeClientPixelPoint,
    ) -> Result<(), NativePlatformFailure>;

    fn move_cursor(&self, screen_point: (i32, i32)) -> Result<(), NativePlatformFailure>;

    fn request_normal_close(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<NormalNativeCloseRequestObservation, NativePlatformFailure>;

    fn verify_process_window_released(&self, process_id: u32) -> Result<(), NativePlatformFailure>;
}

#[cfg(target_os = "windows")]
pub(crate) mod sealed {
    pub trait Sealed {}
}
