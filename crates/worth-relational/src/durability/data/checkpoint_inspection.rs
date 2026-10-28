//! Typed reconstructive inspection; these values confer no recovery authority.
use crate::history::data::RecordAllocationClass;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Physical record coordinates decoded by the native checkpoint owner.
/// Inspection data confers no allocation or mutation authority.
pub struct CheckpointRecordIdentity {
    pub class: RecordAllocationClass,
    pub partition_id: u32,
    pub slot: u64,
    pub generation: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Exclusive append frontier for one physical partition and record class.
/// This diagnostic value grants no reservation authority.
pub struct CheckpointRecordFrontier {
    pub class: RecordAllocationClass,
    pub partition_id: u32,
    pub next_slot: u64,
}

/// Exact original value encodings for all non-exempt native fields. Map framing
/// and record/symbol allocation frontiers or generation rows are not model fields. Unknown fields
/// are refused by the native codec, not silently discarded by this projection.
#[derive(Debug, PartialEq, Eq)]
pub struct NativeCheckpointModelBytes<'a> {
    pub(crate) envelope: BTreeMap<String, &'a [u8]>,
    pub(crate) fields: BTreeMap<String, &'a [u8]>,
    pub(crate) allocator: BTreeMap<String, &'a [u8]>,
}
#[derive(Debug)]
/// Reconstructive native model-byte view, allocator coordinates and live census.
pub struct RelationalCheckpointInspection<'a> {
    pub model: NativeCheckpointModelBytes<'a>,
    /// Provisional symbol IDs can burn without publishing their meanings.
    pub symbol_identity_frontier: u32,
    pub frontiers: Vec<CheckpointRecordFrontier>,
    pub generations: Vec<CheckpointRecordIdentity>,
    pub live_records: BTreeSet<CheckpointRecordIdentity>,
}

impl super::RelationalNativeCheckpoint {
    /// Decode the current checkpoint grammar for diagnostics and atomicity
    /// comparisons. This reconstructs O(B log B) work and O(B) metadata in B
    /// frame bytes, including all encoded retained branch roots. It cannot
    /// restore or mutate a runtime; ordinary reads use branch observations.
    pub fn inspect(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalCheckpointInspection<'_>, super::RelationalRecoveryStop> {
        Self::inspect_bytes(execution, self.bytes())
    }
    /// Inspect an embedded native frame without copying its enclosing buffer.
    pub fn inspect_bytes<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bytes: &'a [u8],
    ) -> Result<RelationalCheckpointInspection<'a>, super::RelationalRecoveryStop> {
        crate::durability::log::native_checkpoint_codec::inspect_checkpoint(execution, bytes)
    }
}
