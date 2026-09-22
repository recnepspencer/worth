#[cfg(worth_ui_certified_executable)]
use std::fmt;
#[cfg(worth_ui_certified_executable)]
use std::time::Instant;

#[cfg(worth_ui_certified_executable)]
use super::NativeInputEnvironmentDenial;
#[cfg(worth_ui_certified_executable)]
use crate::external_observation::{
    NativeClientPixelCapture, NativeClientPixelPoint, NativeInputDeliveryObservation,
    NativeInputProbeKind, NativeWindowVisibilityTransitionObservation,
    NormalNativeCloseRequestObservation, ProcessBoundNativeClientAreaObservation,
};

/// The doctrine posture (milestone-3.10.3 §D5) this build holds. Each variant
/// exists only in the builds where it is the truth, so no build can construct
/// a posture it does not hold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativePlatformPosture {
    /// The native courtroom executes here against a real observer.
    #[cfg(worth_ui_certified_executable)]
    CertifiedExecutable,
    /// The product runs on this target but no observer lane executes yet.
    #[cfg(all(not(worth_ui_certified_executable), worth_ui_product_executable))]
    NotYetCertifiedExecutable,
    /// Every scenario compiles and none executes.
    #[cfg(not(worth_ui_product_executable))]
    CompileOnly,
}

impl NativePlatformPosture {
    pub(crate) fn name(self) -> &'static str {
        match self {
            #[cfg(worth_ui_certified_executable)]
            Self::CertifiedExecutable => "certified_executable",
            #[cfg(all(not(worth_ui_certified_executable), worth_ui_product_executable))]
            Self::NotYetCertifiedExecutable => "not_yet_certified_executable",
            #[cfg(not(worth_ui_product_executable))]
            Self::CompileOnly => "compile_only",
        }
    }
}

#[derive(Debug)]
#[cfg(worth_ui_certified_executable)]
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
    InputEnvironment(NativeInputEnvironmentDenial),
    InputDelivery(String),
    InputDeliveryIndeterminate {
        kind: NativeInputProbeKind,
        delivered_event_count: u32,
        detail: String,
    },
    ProcessWindowResidue(usize),
}

#[cfg(worth_ui_certified_executable)]
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

/// Whether the product's presentation surface succeeds itself between window
/// creation and the first presented frame without the product asking for a
/// change. A succession registers the successor retained target while the
/// predecessor is still live (peak two) and requires one reconstruction of
/// every binding registered at that moment, so the census a lane expects
/// follows from this declaration rather than from a vendor's measurement.
///
/// The Windows lane records one (`_docs/worth-ui/milestone-3.14.1-evidence/
/// p2-world-01.json` peaks `retained_targets` at 2 with a single frame); the
/// X11 lane maps the window at its final basis and records none (Xvfb, 144
/// dpi, 2026-09-22).
#[cfg(worth_ui_certified_executable)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CreationSurfaceSuccession {
    None,
    Once,
}

#[cfg(worth_ui_certified_executable)]
impl CreationSurfaceSuccession {
    /// Peak live retained targets: a successor is registered while its
    /// predecessor is still live, so one succession peaks at two.
    pub(crate) const fn peak_retained_targets(self) -> u64 {
        match self {
            Self::None => 1,
            Self::Once => 2,
        }
    }

    /// Reconstruction requirements a one-binding world accrues: one per
    /// succession, none when the surface is final at creation.
    pub(crate) const fn reconstruction_requirements(self) -> u64 {
        match self {
            Self::None => 0,
            Self::Once => 1,
        }
    }
}

#[cfg(worth_ui_certified_executable)]
#[test]
fn a_creation_time_succession_costs_one_successor_target_and_one_reconstruction() {
    let none = CreationSurfaceSuccession::None;
    let once = CreationSurfaceSuccession::Once;
    assert_eq!(
        (
            none.peak_retained_targets(),
            none.reconstruction_requirements()
        ),
        (1, 0)
    );
    assert_eq!(
        (
            once.peak_retained_targets(),
            once.reconstruction_requirements()
        ),
        (2, 1)
    );
}

#[cfg(worth_ui_certified_executable)]
pub(crate) trait NativePlatformContract: sealed::Sealed {
    type BoundClientArea;

    /// The platform's creation-time surface succession, declared once by the
    /// observer that knows the windowing system.
    const CREATION_SURFACE_SUCCESSION: CreationSurfaceSuccession;

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

    fn move_cursor(&self, screen_point: (i32, i32)) -> Result<(), NativePlatformFailure>;

    fn request_normal_close(
        &self,
        bound: &Self::BoundClientArea,
    ) -> Result<NormalNativeCloseRequestObservation, NativePlatformFailure>;

    fn verify_process_window_released(&self, process_id: u32) -> Result<(), NativePlatformFailure>;
}

#[cfg(worth_ui_certified_executable)]
pub(crate) mod sealed {
    pub trait Sealed {}
}
