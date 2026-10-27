//! Typed reconstructive inspection; these values confer no recovery authority.
use crate::history::data::RecordAllocationClass;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CheckpointRecordIdentity {
    pub class: RecordAllocationClass,
    pub partition_id: u32,
    pub slot: u64,
    pub generation: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointRecordFrontier {
    pub class: RecordAllocationClass,
    pub partition_id: u32,
    pub next_slot: u64,
}

/// Exact original value encodings for all non-exempt native fields. Map framing
/// and allocator frontier/generation rows are not model fields. Unknown fields
/// are refused by the native codec, not silently discarded by this projection.
#[derive(Debug, PartialEq, Eq)]
pub struct NativeCheckpointModelBytes<'a> {
    pub(crate) envelope: BTreeMap<String, &'a [u8]>,
    pub(crate) fields: BTreeMap<String, &'a [u8]>,
    pub(crate) allocator: BTreeMap<String, &'a [u8]>,
}
#[derive(Debug)]
pub struct RelationalCheckpointInspection<'a> {
    pub model: NativeCheckpointModelBytes<'a>,
    pub frontiers: Vec<CheckpointRecordFrontier>,
    pub generations: Vec<CheckpointRecordIdentity>,
    pub live_records: BTreeSet<CheckpointRecordIdentity>,
}

impl super::RelationalNativeCheckpoint {
    /// Decode the current checkpoint grammar for diagnostics and atomicity
    /// comparisons. This is reconstructive O(checkpoint bytes + arena rows)
    /// work, not an ordinary read. It cannot restore or mutate a runtime.
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
        crate::durability::log::native_file_codec::inspect_checkpoint(execution, bytes)
    }
}
