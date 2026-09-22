mod lifecycle_stream;
mod lifecycle_teardown;
#[cfg(worth_ui_certified_executable)]
mod native_client_area;
#[cfg(target_os = "windows")]
mod native_input_bracket;
#[cfg(target_os = "windows")]
#[cfg(worth_ui_certified_executable)]
mod native_input_delivery;
#[cfg(worth_ui_certified_executable)]
mod process_liveness;

pub(crate) use lifecycle_stream::{
    LifecycleFailureSnapshot, LifecycleStreamMeasurement, LifecycleTraceEntry,
    PlatformPulseLifecycleStream, PlatformPulseLifecycleStreamFailure,
};
pub(crate) use lifecycle_teardown::{
    PlatformPulseLifecycleTeardownEvidence, PlatformPulseLifecycleTeardownFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use native_client_area::{
    NativeClientAreaBounds, NativeClientPixelCapture, NativeClientPixelPoint, NativeWindowIdentity,
    NativeWindowVisibilityTransitionMechanism, NativeWindowVisibilityTransitionObservation,
    NormalNativeCloseRequestObservation, ProcessBoundNativeClientAreaObservation,
};
#[cfg(target_os = "windows")]
pub(crate) use native_input_bracket::NativeInputDeliveryTiming;
#[cfg(target_os = "windows")]
#[cfg(worth_ui_certified_executable)]
pub(crate) use native_input_delivery::{
    NativeInputDeliveryObservation, NativeInputProbeKind, NativeKeyboardCommand,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use process_liveness::{
    begin_stable_process_liveness, StableProcessLivenessFailure, StableProcessLivenessObservation,
};
