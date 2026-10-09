//! Controls and head replay retained for one authenticated released edge.

use worth_store_physical_format::{BlobReclaimDescriptorV3, DropSetManifestV3};
use worth_store_recovery_physics::{
    VerifiedAddressedReleasedControlFrame, VerifiedOrderedReleasedHeadReplayV14,
};

#[derive(Debug)]
pub(in crate::orchestration::planning) struct OrderedReleasedObservation {
    pub(in crate::orchestration::planning) operation: [u8; 32],
    pub(in crate::orchestration::planning) descriptor: BlobReclaimDescriptorV3,
    pub(in crate::orchestration::planning) manifest: DropSetManifestV3,
    pub(in crate::orchestration::planning) candidate_root_generation: u64,
    pub(in crate::orchestration::planning) descriptor_frame: VerifiedAddressedReleasedControlFrame,
    pub(in crate::orchestration::planning) reservation_frame: VerifiedAddressedReleasedControlFrame,
    pub(in crate::orchestration::planning) manifest_frame: VerifiedAddressedReleasedControlFrame,
    pub(in crate::orchestration::planning) head_replay: VerifiedOrderedReleasedHeadReplayV14,
}
