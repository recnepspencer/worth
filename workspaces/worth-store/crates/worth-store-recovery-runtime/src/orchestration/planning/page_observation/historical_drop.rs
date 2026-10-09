//! Selected V3 result custody for old WAL images removed by that drop.
//! Classification does not itself authorize a Serving seal.

use std::sync::Arc;
use worth_store_physical_format::{
    BlobReclaimDescriptorV3, DropSetManifestV3, PersistedRecordIdentity,
};
use worth_store_recovery_physics::VerifiedOrderedRootHistory;

#[path = "historical_drop/classification.rs"]
mod classification;
pub(super) use classification::{classify, ClassifiedTargets, HistoryWalk};
#[path = "historical_drop/media_controls.rs"]
mod media_controls;
use media_controls::target_record;
pub(super) use media_controls::{selected_control, source_root};
#[path = "historical_drop/ordered_retirements.rs"]
mod ordered_retirements;
#[path = "historical_drop/released_drops.rs"]
mod released_drops;
#[path = "historical_drop/retired_targets.rs"]
mod retired_targets;
#[cfg(test)]
#[path = "historical_drop/retirement_fixture.rs"]
mod retirement_fixture;

#[derive(Debug, Clone)]
pub(in crate::orchestration::planning) struct HistoricalDropEvidence {
    pub(in crate::orchestration::planning) operation: [u8; 32],
    pub(in crate::orchestration::planning) descriptor_record: PersistedRecordIdentity,
    pub(in crate::orchestration::planning) descriptor: BlobReclaimDescriptorV3,
    pub(in crate::orchestration::planning) manifest: DropSetManifestV3,
    pub(in crate::orchestration::planning) ordered_history: Arc<VerifiedOrderedRootHistory>,
}
