use serde::{Deserialize, Serialize};

/// Native generation selection work, separate from entry lookup and index construction.
/// Ordered-map key comparisons are O(log retained generations); selecting one exact
/// generation reads one payload. Cold inventories charge every enumerated payload.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexSelectionCounters {
    pub generation_payload_reads: usize,
    pub history_inventory_entries: usize,
}
