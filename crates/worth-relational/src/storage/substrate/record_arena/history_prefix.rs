//! Address of the earlier metadata rows omitted from a resident Head suffix.
mod custody;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::identity::data::PartitionId;
pub(crate) use custody::RecordHistoryCustody;
use std::sync::Arc;

/// This reference follows metadata history, including across generation reuse.
/// It owns only a read port and immutable placement facts; the Full demand
/// orchestrator owns decoding, schema authority and symbol admission.
#[derive(Debug, Clone)]
pub(crate) struct RecordHistoryPrefix {
    pub(crate) custody: Arc<RecordHistoryCustody>,
    pub(crate) member: RelationalDurableMemberLocator,
    pub(crate) history_members: Arc<crate::durability::data::RelationalHistoryMemberDirectory>,
    pub(crate) partition_id: PartitionId,
    pub(crate) schema_carrier_digest: [u8; 32],
    pub(crate) row_index: usize,
    pub(crate) history_opening:
        crate::storage::substrate::StorageAllocation<super::RecordHistoryOpening>,
}

impl RecordHistoryPrefix {
    pub(crate) fn source(
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
