mod bounded_loader;
mod loaded_frame;
mod read_source;
#[cfg(feature = "certification-test-authority")]
mod speculative;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) enum ExactFrameSourceExtent {
    #[cfg(feature = "certification-test-authority")]
    CoordinateOnly,
    CompleteArtifact(std::num::NonZeroU64),
    /// A published route bounds the frame; an arena may grow independently.
    ArenaRange(worth_store_physical_format::ExtentArenaRange),
}

pub(in crate::physical_runtime::record_serving) use super::frame_load_failure::{
    FrameLoadFailure, FrameLoadFailureKind, FrameLoadFaultCause,
};
pub(in crate::physical_runtime::record_serving) use bounded_loader::BoundedFrameLoader;
pub(in crate::physical_runtime::record_serving) use loaded_frame::LoadedPhysicalFrame;
pub(in crate::physical_runtime::record_serving) use loaded_frame::PhysicalFrameAccessOrigin;
pub(in crate::physical_runtime::record_serving::residency) use read_source::DirectFrameReadSource;
pub(in crate::physical_runtime::record_serving) use read_source::{
    CanonicalFrameReadSource, FrameLoadPort,
};
