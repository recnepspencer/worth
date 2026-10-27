use crate::query::data::{PlannedQueryPacket, ReadPacketPlan};
use crate::runtime::PartitionEdition;
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;
use crate::storage::data::{
    ChunkDiagnostics, ChunkedStorageSummary, PartitionStorageStats, RecordLifecycleState,
    StorageStats,
};
use crate::storage::substrate::RecordKind;

pub struct StorageAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub fn storage_access(&self) -> StorageAccess<'_> {
        StorageAccess::new(self)
    }
}

impl<'runtime> StorageAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    /// Pin one edition of the authoritative substrate for a whole traversal.
    ///
    /// Callers must hold the returned edition for the entire read rather than
    /// re-acquiring per record or per slot: re-acquisition is what turns a
    /// bounded read into source-breadth work.
    pub(crate) fn current_edition(&self) -> PartitionEdition {
        self.runtime.acquire_partition_edition()
    }

    pub(crate) fn entity_slot_count(&self) -> usize {
        self.current_edition()
            .cells()
            .map(|(_, region)| region.cell().slot_counts().0)
            .sum()
    }

    pub(crate) fn relation_slot_count(&self) -> usize {
        self.current_edition()
            .cells()
            .map(|(_, region)| region.cell().slot_counts().1)
            .sum()
    }

    pub(crate) fn entity_chunk_size(&self) -> usize {
        self.runtime.config.storage.layout.entity_chunk_size.max(1)
    }

    pub(crate) fn relation_chunk_size(&self) -> usize {
        self.runtime
            .config
            .storage
            .layout
            .relation_chunk_size
            .max(1)
    }

    pub fn partition_ids(&self) -> Vec<crate::identity::data::PartitionId> {
        crate::storage::partition::storage_stats::partition_ids(self.runtime)
    }

    /// Demand the whole mirror before reporting partition statistics.
    pub fn partition_storage_stats(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<PartitionStorageStats>, crate::execution::RelationalExecutionDenialCause> {
        let edition = self
            .current_edition()
            .demand_all(execution, &self.runtime.services.symbols)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        let answer = crate::storage::partition::storage_stats::partition_storage_stats(
            self.runtime,
            &edition,
        );
        crate::execution::check_request(execution)?;
        Ok(answer)
    }

    /// Demand the whole mirror before reporting its storage and visibility statistics.
    pub fn storage_stats(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<StorageStats, crate::execution::RelationalExecutionDenialCause> {
        let edition = self
            .current_edition()
            .demand_all(execution, &self.runtime.services.symbols)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        let answer =
            crate::storage::partition::storage_stats::storage_stats(self.runtime, &edition);
        crate::execution::check_request(execution)?;
        Ok(answer)
    }

    pub fn chunked_storage_summary(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: crate::identity::data::VersionId,
    ) -> Result<ChunkedStorageSummary, crate::execution::RelationalExecutionDenialCause> {
        let edition = self
            .current_edition()
            .demand_all(execution, &self.runtime.services.symbols)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        let answer = crate::storage::partition::chunks::chunked_storage_summary(
            self.runtime,
            &edition,
            version_id,
        );
        crate::execution::check_request(execution)?;
        Ok(answer)
    }

    pub fn chunk_diagnostics(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: crate::identity::data::VersionId,
    ) -> Result<ChunkDiagnostics, crate::execution::RelationalExecutionDenialCause> {
        let edition = self
            .current_edition()
            .demand_all(execution, &self.runtime.services.symbols)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        let answer = crate::storage::partition::chunks::chunk_diagnostics(
            self.runtime,
            &edition,
            version_id,
        );
        crate::execution::check_request(execution)?;
        Ok(answer)
    }

    pub fn plan_read_explicit_query_packet(
        &self,
        handle: &SnapshotHandle,
        packet: &PlannedQueryPacket,
    ) -> Option<ReadPacketPlan> {
        crate::storage::partition::chunks::plan_read_explicit_query_packet(
            self.runtime,
            handle,
            packet,
        )
    }

    pub fn outgoing_relations_for_entity(
        &self,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<crate::identity::data::RelationId> {
        crate::storage::partition::adjacency_queries::outgoing_relations_for_entity(
            self.runtime,
            entity_id,
            version_id,
        )
    }

    pub fn incoming_relations_for_entity(
        &self,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<crate::identity::data::RelationId> {
        crate::storage::partition::adjacency_queries::incoming_relations_for_entity(
            self.runtime,
            entity_id,
            version_id,
        )
    }

    pub fn all_relations_for_entity(
        &self,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<crate::identity::data::RelationId> {
        crate::storage::partition::adjacency_queries::all_relations_for_entity(
            self.runtime,
            entity_id,
            version_id,
        )
    }
}

/// A partition's occupied slots, read out of an edition the caller pinned.
///
/// Slot-by-slot loops must go through this rather than the acquiring twin: a
/// read-only sweep over S slots should acquire the substrate once, not S times.
pub(crate) fn record_slots_in<K: RecordKind>(
    edition: &impl crate::storage::overlay::PartitionAccess,
    partition_id: crate::identity::data::PartitionId,
) -> Vec<usize> {
    edition
        .get_partition(partition_id)
        .map(|partition| K::arena(partition).occupied_slots())
        .unwrap_or_default()
}

/// One slot's retention surface, read out of an edition the caller pinned.
///
/// Retention mutations use a resident writer entry instead; an inspection
/// holds its immutable resident edition for the entire traversal.
pub(crate) fn record_slot_surface_in<K: RecordKind>(
    edition: &impl crate::storage::overlay::PartitionAccess,
    partition_id: crate::identity::data::PartitionId,
    slot: usize,
) -> Option<RecordSlotSurface> {
    let partition = edition.get_partition(partition_id)?;
    let arena = K::arena(partition);
    let slot_view = arena.get_slot(slot)?;
    Some(RecordSlotSurface {
        snapshot_pins: arena.snapshot_pin_count(slot).unwrap_or(0),
        branch_pins: arena.branch_pin_count(slot).unwrap_or(0),
        replay_pins: arena.replay_pin_count(slot).unwrap_or(0),
        lifecycle: slot_view.lifecycle(),
    })
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RecordSlotSurface {
    pub(crate) snapshot_pins: u32,
    pub(crate) branch_pins: u32,
    pub(crate) replay_pins: u32,
    pub(crate) lifecycle: RecordLifecycleState,
}
