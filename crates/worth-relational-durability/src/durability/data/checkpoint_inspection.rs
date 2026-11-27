//! Typed reconstructive inspection; these values confer no recovery authority.
use contracts::history::data::record_allocation_class::RecordAllocationClass;
use std::collections::{BTreeMap, BTreeSet};
use worth_relational_contracts::internal_port as contracts;
pub mod model_comparison;

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

/// Exact original value encodings for the exempt monotonic record-allocation
/// fields (`append_frontiers`, `generation_high_water`, `partition_high_water`,
/// and `pages`). Excluded from `NativeCheckpointModelBytes` so model-invariance
/// checks allow burned monotonic allocations, while full-invariance checks can
/// still detect any record allocation on bare `CHK24` frames without an external
/// cold member source.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct NativeRecordAllocationBytes<'a> {
    pub(crate) append_frontiers: &'a [u8],
    pub(crate) generation_high_water: &'a [u8],
    pub(crate) partition_high_water: &'a [u8],
    pub(crate) pages: &'a [u8],
}

#[derive(Debug)]
/// Reconstructive native model-byte view, allocator coordinates and live census.
pub struct RelationalCheckpointInspection<'a> {
    pub(in crate::internal_port::durability) branch_cells:
        Vec<contracts::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint>,
    pub model: NativeCheckpointModelBytes<'a>,
    pub record_allocation_bytes: NativeRecordAllocationBytes<'a>,
    /// Provisional symbol IDs can burn without publishing their meanings.
    pub symbol_identity_frontier: u32,
    pub frontiers: Vec<CheckpointRecordFrontier>,
    pub generations: Vec<CheckpointRecordIdentity>,
    pub current_publication_records: BTreeSet<CheckpointRecordIdentity>,
    pub mirror_live_records: BTreeSet<CheckpointRecordIdentity>,
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
    ) -> Result<
        RelationalCheckpointInspection<'_>,
        contracts::durability::data::recovery_stop::RelationalRecoveryStop,
    > {
        Self::inspect_bytes(execution, self.bytes())
    }
    /// Inspect a native checkpoint frame without copying its enclosing buffer.
    pub fn inspect_bytes<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bytes: &'a [u8],
    ) -> Result<
        RelationalCheckpointInspection<'a>,
        contracts::durability::data::recovery_stop::RelationalRecoveryStop,
    > {
        crate::durability::log::native_checkpoint_codec::inspect_checkpoint(execution, bytes, None)
    }
    /// Inspect an external-members native checkpoint frame together with its cold member source.
    pub fn inspect_bytes_with_source<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bytes: &'a [u8],
        source: std::sync::Arc<
            dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource,
        >,
    ) -> Result<
        RelationalCheckpointInspection<'a>,
        contracts::durability::data::recovery_stop::RelationalRecoveryStop,
    > {
        crate::durability::log::native_checkpoint_codec::inspect_checkpoint(
            execution,
            bytes,
            Some(source),
        )
    }
}
