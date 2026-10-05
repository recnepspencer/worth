//! Exact issued-snapshot output probes without projection or schema copies.

use worth_foundational::facade::AspectKey;

use crate::identity::data::{EntityId, KindId};
use crate::snapshots::data::SnapshotHandle;
use crate::storage::overlay::PartitionAccess;

use super::VisibilityReadContext;

impl VisibilityReadContext<'_> {
    /// An issued-root liveness/kind probe that distinguishes an unavailable
    /// root (outer None) from a missing, retired, or reused entity (inner
    /// None). No schema record or owned branch basis is materialized.
    pub fn exact_snapshot_live_entity_kind_status(
        &self,
        handle: &SnapshotHandle,
        entity: EntityId,
    ) -> Option<Option<KindId>> {
        if handle.runtime_instance_id != self.runtime.runtime_instance_id() {
            return None;
        }
        let root = self
            .runtime
            .visibility
            .selected_snapshot_root(handle.snapshot_id)?;
        let kind = root
            .get_partition(entity.partition_id)
            .and_then(|partition| partition.entity_arena.get(&entity))
            .and_then(|slot| slot.is_live().then(|| slot.kind_id()).flatten());
        Some(kind)
    }

    /// The kind of a live entity at the root retained by this issued handle.
    /// A retired, reused, foreign, or released handle is unavailable.
    pub fn exact_snapshot_live_entity_kind(
        &self,
        handle: &SnapshotHandle,
        entity: EntityId,
    ) -> Option<KindId> {
        self.exact_snapshot_live_entity_kind_status(handle, entity)
            .flatten()
    }

    /// Native aspect revision at the same issued exact root. An outer `None`
    /// refuses stale identity or unavailable snapshot provenance.
    pub fn exact_snapshot_entity_aspect_version(
        &self,
        handle: &SnapshotHandle,
        entity: EntityId,
        aspect: &AspectKey,
    ) -> Option<Option<u64>> {
        if handle.runtime_instance_id != self.runtime.runtime_instance_id() {
            return None;
        }
        let root = self
            .runtime
            .visibility
            .selected_snapshot_root(handle.snapshot_id)?;
        let partition = root.get_partition(entity.partition_id)?;
        let slot = partition.entity_arena.get(&entity)?;
        if !slot.is_live() {
            return None;
        }
        let symbol = self
            .runtime
            .services
            .symbols
            .with_read(|symbols| symbols.symbol(aspect.as_str()));
        let versions = partition
            .entity_arena
            .aspect_versions_at(entity.slot_index())?;
        Some(symbol.and_then(|symbol| versions.get(&symbol).copied()))
    }
}
