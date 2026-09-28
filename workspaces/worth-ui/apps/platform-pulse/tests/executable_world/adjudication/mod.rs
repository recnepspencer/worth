#[cfg(worth_ui_certified_executable)]
mod appearance_pixels;
#[cfg(worth_ui_certified_executable)]
pub(crate) mod dashboard_visual_oracle;
#[cfg(worth_ui_certified_executable)]
mod lifecycle_cleanup;
#[cfg(worth_ui_certified_executable)]
mod native_color;
#[cfg(target_os = "windows")]
mod scroll_chrome_pixel_failure;
#[cfg(target_os = "windows")]
mod scroll_chrome_pixels;
#[cfg(target_os = "windows")]
#[cfg(worth_ui_certified_executable)]
mod source_to_pixel;
mod visual_contract_manifest;

#[cfg(worth_ui_certified_executable)]
pub(crate) use appearance_pixels::{
    adjudicate_first_frame_appearance, FirstFrameAppearanceFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use lifecycle_cleanup::{
    adjudicate_lifecycle_cleanup, CausalLifecycleCleanupObservationSet,
    ExecutableLifecycleCleanupEvidence, ExecutableLifecycleCleanupFailure,
};
#[cfg(worth_ui_certified_executable)]
pub(crate) use native_color::{
    adjudicate_native_color, ExpectedNativeColor, NativeColorFailure, NativeColorVerdict,
};
#[cfg(target_os = "windows")]
pub(crate) use scroll_chrome_pixel_failure::ScrollChromePixelFailure;
#[cfg(target_os = "windows")]
pub(crate) use scroll_chrome_pixels::{
    adjudicate_content_shift, adjudicate_vertical_thumb, notch_points, physical_px,
    platform_wheel_lines_per_notch, ContentShiftEvidence, RecentActivityScrollGeometry,
    VerticalThumbEvidence,
};
#[cfg(target_os = "windows")]
#[cfg(worth_ui_certified_executable)]
pub(crate) use source_to_pixel::{
    adjudicate_first_frame, adjudicate_source_signal_first_frame, CausalFirstFrameObservationSet,
    ExecutableFirstFrameEvidence, ExecutableFirstFrameFailure,
};
