//! Reclaim source and drop records are representation, not permission to drop.

mod basis;
mod descriptor;
mod descriptor_v2;
mod descriptor_v3;
mod drop_set_manifest;
mod drop_set_manifest_v2;
mod drop_set_manifest_v3;
mod original_drop_reserved;
mod source_basis;

pub use basis::FailedIngestReclaimBasisV1;
pub use descriptor::BlobReclaimDescriptorV1;
pub use descriptor_v2::{BlobReclaimDescriptorV2, ReleasedDropPredecessorV1};
pub use descriptor_v3::{BlobReclaimDescriptorV3, ReleasedDropCustodyV1};
pub use drop_set_manifest::{DropSetManifestV1, MAXIMUM_DROP_SET_RECORDS};
pub use drop_set_manifest_v2::DropSetManifestV2;
pub use drop_set_manifest_v3::{DropSetManifestV3, DropSetManifestV3View};
pub use original_drop_reserved::{OriginalDropReservationRequestV1, OriginalDropReservedV1};
pub use source_basis::{
    BlobReclaimSourceBasisV1, BlobReclaimSourceKind, ReleasedGenerationReclaimBasisV1,
};

#[cfg(test)]
mod release_tests;
#[cfg(test)]
mod tests;
