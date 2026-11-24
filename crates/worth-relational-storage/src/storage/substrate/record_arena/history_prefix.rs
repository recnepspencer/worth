//! Address of the earlier metadata rows omitted from a resident Head suffix.
pub mod custody;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::identity::data::PartitionId;
pub use custody::RecordHistoryCustody;
use std::sync::Arc;

/// This reference follows metadata history, including across generation reuse.
/// It owns only a read port and immutable placement facts; the Full demand
/// orchestrator owns decoding, schema authority and symbol admission.
#[derive(Debug, Clone)]
pub struct RecordHistoryPrefix {
    pub custody: Arc<RecordHistoryCustody>,
    pub member: RelationalDurableMemberLocator,
    pub history_members:
        Arc<crate::storage::data::history_member_directory::RelationalHistoryMemberDirectory>,
    pub partition_id: PartitionId,
    pub schema_carrier_digest: [u8; 32],
    pub row_index: usize,
    pub history_opening: crate::storage::substrate::StorageAllocation<super::RecordHistoryOpening>,
}

impl RecordHistoryPrefix {
    pub fn source(
        &self,
    ) -> Result<
        Arc<dyn RelationalDurableColdSource>,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        self.custody
            .source()
            .ok_or(crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable)
    }
}
