//! Request-aware point probes at an exact owner-issued snapshot.
use super::VisibilityReadContext;
use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::{EntityId, KindId};
use crate::snapshots::data::SnapshotHandle;
use worth_foundational::facade::AspectKey;
impl VisibilityReadContext<'_> {
    /// An unavailable handle differs from an absent or retired entity. Cold
    /// storage failure is neither and is returned before borrowing the slot.
    pub fn read_exact_snapshot_live_entity_kind_status(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        entity: EntityId,
    ) -> Result<Option<Option<KindId>>, RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        if handle.runtime_instance_id != self.runtime.runtime_instance_id() {
            return Ok(None);
        }
        let Some(root) = self
            .runtime
            .visibility
            .selected_snapshot_root(handle.snapshot_id)
        else {
            return Ok(None);
        };
        root.demand_regions_head(
            execution,
            &self.runtime.services.symbols,
            [entity.partition_id],
        )?;
        let kind = root
            .admitted_head_region(entity.partition_id)
            .and_then(|partition| partition.entity_arena().get(&entity))
            .and_then(|slot| slot.is_live().then(|| slot.kind_id()).flatten());
        Ok(Some(kind))
    }
    /// The live kind at this exact selected root, with storage denials retained.
    pub fn read_exact_snapshot_live_entity_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        entity: EntityId,
    ) -> Result<Option<KindId>, RelationalExecutionDenialCause> {
        self.read_exact_snapshot_live_entity_kind_status(execution, handle, entity)
            .map(Option::flatten)
    }
    /// Revision of one aspect at the selected exact root. The outer option
    /// rejects stale identity or missing snapshot provenance, never cold data.
    pub fn read_exact_snapshot_entity_aspect_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        entity: EntityId,
        aspect: &AspectKey,
    ) -> Result<Option<Option<u64>>, RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        if handle.runtime_instance_id != self.runtime.runtime_instance_id() {
            return Ok(None);
        }
        let Some(root) = self
            .runtime
            .visibility
            .selected_snapshot_root(handle.snapshot_id)
        else {
            return Ok(None);
        };
        root.demand_regions_head(
            execution,
            &self.runtime.services.symbols,
            [entity.partition_id],
        )?;
        let Some(partition) = root.admitted_head_region(entity.partition_id) else {
            return Ok(None);
        };
        let Some(slot) = partition.entity_arena().get(&entity) else {
            return Ok(None);
        };
        if !slot.is_live() {
            return Ok(None);
        }
        let symbol = self
            .runtime
            .services
            .symbols
            .with_read(|symbols| symbols.symbol(aspect.as_str()));
        let Some(versions) = partition
            .entity_arena()
            .aspect_versions_at(entity.slot_index())
        else {
            return Ok(None);
        };
        Ok(Some(
            symbol.and_then(|symbol| versions.get(&symbol).copied()),
        ))
    }
}
