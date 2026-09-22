mod lifecycle_stream;
mod lifecycle_teardown;
#[cfg(target_os = "windows")]
mod native_client_area;
#[cfg(target_os = "windows")]
mod native_frame_timing;
#[cfg(target_os = "windows")]
mod native_input_delivery;
#[cfg(target_os = "windows")]
mod process_liveness;

pub(crate) use lifecycle_stream::{
    LifecycleFailureSnapshot, LifecycleStreamMeasurement, LifecycleTraceEntry,
    PlatformPulseLifecycleStream, PlatformPulseLifecycleStreamFailure,
};
pub(crate) use lifecycle_teardown::{
    PlatformPulseLifecycleTeardownEvidence, PlatformPulseLifecycleTeardownFailure,
};
#[cfg(target_os = "windows")]
pub(crate) use native_client_area::{
    NativeClientAreaBounds, NativeClientPixelCapture, NativeClientPixelPoint, NativeWindowIdentity,
    NativeWindowVisibilityTransitionObservation, NormalNativeCloseRequestObservation,
    ProcessBoundNativeClientAreaObservation,
};
#[cfg(target_os = "windows")]
pub(crate) use native_frame_timing::{NativeInputDeliveryTiming, NativeTimedClientPixelCapture};
#[cfg(target_os = "windows")]
pub(crate) use native_input_delivery::{
    NativeInputDeliveryObservation, NativeInputProbeKind, NativeKeyboardCommand,
};
#[cfg(target_os = "windows")]
pub(crate) use process_liveness::{
    begin_stable_process_liveness, StableProcessLivenessFailure, StableProcessLivenessObservation,
};
