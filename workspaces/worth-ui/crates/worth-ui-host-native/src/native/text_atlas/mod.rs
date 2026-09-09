//! Native-owned bounded atlas transaction and resource lifecycle.

mod admission;
mod alpha;
mod candidate_store;
mod capacity;
mod census;
mod cleanup;
mod color;
mod demand;
mod demand_admission;
mod entry;
#[cfg(test)]
mod eviction;
mod image_observation;
#[cfg(test)]
mod image_observation_tests;
mod in_flight;
mod key;
mod ownership;
mod pinning;
mod placement;
mod plan_observation;
mod planning;
mod raster_upload;
mod recovery;
mod settlement;
mod settling;
#[cfg(test)]
mod test_device_tests;
mod transaction;
#[cfg(test)]
mod transaction_model_assertion;
mod transaction_plan_snapshot;
mod upload;
mod upload_staging;

#[cfg(test)]
mod boundary_tests;
#[cfg(test)]
mod content_extent_tests;
#[cfg(test)]
mod correlation_tests;
#[cfg(test)]
pub(crate) mod eviction_tests;
#[cfg(test)]
mod model_key;
#[cfg(test)]
mod model_oracle;
#[cfg(test)]
mod model_placement;
#[cfg(test)]
mod model_records;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod ownership_tests;
#[cfg(test)]
mod pinning_capacity_tests;
#[cfg(test)]
mod placement_model_tests;
#[cfg(test)]
mod recovery_identity_tests;

pub use capacity::UiNativeTextAtlasQualifiedCapacity;
pub use census::UiNativeTextAtlasCensus;
pub(crate) use census::{UiNativeTextAtlasPhysicalPosture, UiNativeTextAtlasResourceClass};
pub(crate) use demand::UiNativeTextAtlasDemand;
pub(crate) use image_observation::UiNativeTextAtlasImageObservation;
pub(crate) use in_flight::UiNativeTextAtlasInFlight;
pub(crate) use key::canonical_raster_key_bytes;
pub use key::UiAtlasEntryIdentity;
pub use ownership::UiNativeTextPinObservation;
pub(crate) use ownership::{UiNativeTextAtlas, UiNativeTextAtlasEntryView};
pub use plan_observation::UiNativeTextAtlasPlanObservation;
pub(crate) use raster_upload::UiNativeTextAtlasUpload;
#[cfg(test)]
pub(crate) use recovery::UiNativeTextAtlasGeneration;
pub use recovery::{UiNativeTextAtlasDenial, UiNativeTextAtlasRecovery};
pub(crate) use settlement::UiNativeTextAtlasCommitOutcome;
#[cfg(test)]
pub(crate) use test_device_tests::qualified_test_device;
pub(crate) use transaction::{
    UiNativeTextAtlasExternalOutcome, UiNativeTextAtlasPinRequest, UiNativeTextAtlasPinTransition,
    UiNativeTextAtlasTransactionPlan,
};
#[cfg(test)]
pub(crate) use transaction_model_assertion::assert_independent_committed_transaction;
pub(crate) use upload::UiNativeGpuAtlasKind;
#[cfg(test)]
pub(crate) use upload::UiNativeTextAtlasGpuUploadRequest;
pub(crate) use upload::{UiNativeTextAtlasGpuBatchUpload, UiNativeTextAtlasGpuPages};
