//! Private runtime join between text-owned raster meaning and host-owned atlas
//! effects.
//!
//! The runtime derives exact demands and rasterizes only the misses selected
//! by the host transaction. Physical Signal progression and atlas ownership
//! remain inside the native host; this module retains only portable demand,
//! pin-candidate, and settlement meaning.

mod async_correspondence;
mod mounted_coordinator;
mod preparation;
mod query_correspondence;
mod rasterization;
mod transaction;
mod work_observation;

pub(crate) use async_correspondence::{
    UiPresentationAsyncPresentedAdmission, UiPresentationAsyncRuntime,
    UiPresentationAsyncTerminalCleanup,
};
pub(crate) use mounted_coordinator::{
    UiMountedTextForegroundReuseUpdate, UiNativeMountedSurfaceTextObservation,
    UiNativeMountedTextCoordinator,
};
pub(super) use preparation::mounted_semantic_text;
pub(crate) use preparation::{
    prepare_complete_semantic_text, prepare_from_foreground_reuse, prepare_mounted_semantic_text,
    presentation_damage_digest, UiMountedEventTimeDpiAuthority,
    UiNativeTextPresentationPreparation, UiNativeTextPresentationPrepared,
    UiNativeTextPresentationReadiness,
};
pub(crate) use query_correspondence::derive_text_presentation_request_bases;
pub(crate) use rasterization::UiNativeTextMissRasterizer;
pub(crate) use transaction::UiNativeTextAtlasTransaction;
pub(crate) use work_observation::{
    UiNativeTextPresentationMechanicObservation, UiNativeTextPresentationWorkObservation,
};

#[cfg(test)]
mod complete_demand_tests;

#[cfg(test)]
mod foreground_coverage_test_world;
#[cfg(test)]
mod foreground_coverage_tests;
#[cfg(test)]
mod foreground_lifecycle_tests;
#[cfg(test)]
mod foreground_reconstruction_tests;
#[cfg(test)]
mod foreground_replay_tests;
#[cfg(test)]
mod physical_replay_tests;
