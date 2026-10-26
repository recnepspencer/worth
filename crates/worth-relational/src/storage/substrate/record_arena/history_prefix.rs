//! Address of the earlier metadata rows omitted from a resident Head suffix.
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::identity::data::PartitionId;
use std::sync::Arc;

/// This reference follows metadata history, including across generation reuse.
/// It owns only a read port and immutable placement facts; the Full demand
/// orchestrator owns decoding, schema authority and symbol admission.
#[derive(Debug, Clone)]
pub(crate) struct RecordHistoryPrefix {
    pub(crate) source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) member: RelationalDurableMemberLocator,
    pub(crate) partition_id: PartitionId,
    pub(crate) schema_carrier_digest: [u8; 32],
    pub(crate) row_index: usize,
}
