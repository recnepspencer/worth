use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use dashmap::DashMap;

use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::validation::engine::state_view::slot_resolution::AspectLocation;
use crate::validation::engine::state_view::{VisibleEntityMetadata, VisibleRelationMetadata};

use super::plan::{CandidateInputBasis, SharedCandidateInputs};

// A map guard is held only long enough to find the per-key cell. The read
// executes outside the shard lock, while the cell still admits it exactly once.
fn read_once<K: Eq + Hash, V: Clone>(
    map: &DashMap<K, Arc<OnceLock<V>>>,
    key: K,
    read: impl FnOnce() -> V,
    physical_reads: &AtomicUsize,
    reuse_hits: &AtomicUsize,
) -> V {
    let cell = if let Some(entry) = map.get(&key) {
        Arc::clone(entry.value())
    } else {
        map.entry(key).or_default().clone()
    };
    let mut initialized = false;
    let value = cell
        .get_or_init(|| {
            initialized = true;
            physical_reads.fetch_add(1, Ordering::Relaxed);
            read()
        })
        .clone();
    if !initialized {
        reuse_hits.fetch_add(1, Ordering::Relaxed);
    }
    value
}

impl SharedCandidateInputs {
    pub(crate) fn touched_partitions(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        read: impl FnOnce() -> Option<Vec<PartitionId>>,
    ) -> Option<Arc<[PartitionId]>> {
        let gather = || read().map(Arc::from);
        if !self.sharing_enabled {
            self.entries
                .touched_partition_gathers
                .fetch_add(1, Ordering::Relaxed);
            return gather();
        }
        read_once(
            &self.entries.touched_partitions,
            (basis, version),
            gather,
            &self.entries.touched_partition_gathers,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn touched_entity_slots(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        partition: PartitionId,
        read: impl FnOnce() -> Option<Vec<usize>>,
    ) -> Option<Arc<[usize]>> {
        let gather = || read().map(Arc::from);
        if !self.sharing_enabled {
            self.entries
                .touched_entity_slot_gathers
                .fetch_add(1, Ordering::Relaxed);
            return gather();
        }
        read_once(
            &self.entries.touched_entity_slots,
            (basis, version, partition),
            gather,
            &self.entries.touched_entity_slot_gathers,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn touched_relation_slots(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        partition: PartitionId,
        read: impl FnOnce() -> Option<Vec<usize>>,
    ) -> Option<Arc<[usize]>> {
        let gather = || read().map(Arc::from);
        if !self.sharing_enabled {
            self.entries
                .touched_relation_slot_gathers
                .fetch_add(1, Ordering::Relaxed);
            return gather();
        }
        read_once(
            &self.entries.touched_relation_slots,
            (basis, version, partition),
            gather,
            &self.entries.touched_relation_slot_gathers,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn touched_entities(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        gather: impl FnOnce() -> Vec<EntityId>,
    ) -> Arc<[EntityId]> {
        let gather = || Arc::from(gather());
        if !self.sharing_enabled {
            self.entries
                .touched_entity_gathers
                .fetch_add(1, Ordering::Relaxed);
            return gather();
        }
        read_once(
            &self.entries.touched_entities,
            (basis, version),
            gather,
            &self.entries.touched_entity_gathers,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn touched_relations(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        gather: impl FnOnce() -> Vec<RelationId>,
    ) -> Arc<[RelationId]> {
        let gather = || Arc::from(gather());
        if !self.sharing_enabled {
            self.entries
                .touched_relation_gathers
                .fetch_add(1, Ordering::Relaxed);
            return gather();
        }
        read_once(
            &self.entries.touched_relations,
            (basis, version),
            gather,
            &self.entries.touched_relation_gathers,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn entity_aspect(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        locate: impl FnOnce() -> Option<AspectLocation>,
    ) -> Option<AspectLocation> {
        if !self.sharing_enabled {
            self.entries
                .entity_aspect_reads
                .fetch_add(1, Ordering::Relaxed);
            return locate();
        }
        read_once(
            &self.entries.entity_aspects,
            (basis, version, id),
            locate,
            &self.entries.entity_aspect_reads,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn relation_aspect(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: RelationId,
        locate: impl FnOnce() -> Option<AspectLocation>,
    ) -> Option<AspectLocation> {
        if !self.sharing_enabled {
            self.entries
                .relation_aspect_reads
                .fetch_add(1, Ordering::Relaxed);
            return locate();
        }
        read_once(
            &self.entries.relation_aspects,
            (basis, version, id),
            locate,
            &self.entries.relation_aspect_reads,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn adjacency_count(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        outgoing: bool,
        read: impl FnOnce() -> usize,
    ) -> usize {
        if !self.sharing_enabled {
            self.entries
                .adjacency_count_reads
                .fetch_add(1, Ordering::Relaxed);
            return read();
        }
        read_once(
            &self.entries.adjacency_counts,
            (basis, version, id, outgoing),
            read,
            &self.entries.adjacency_count_reads,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn entity(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        read: impl FnOnce() -> Option<VisibleEntityMetadata>,
    ) -> Option<VisibleEntityMetadata> {
        if !self.sharing_enabled {
            self.entries.entity_reads.fetch_add(1, Ordering::Relaxed);
            return read();
        }
        read_once(
            &self.entries.entities,
            (basis, version, id),
            read,
            &self.entries.entity_reads,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn relation(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: RelationId,
        read: impl FnOnce() -> Option<VisibleRelationMetadata>,
    ) -> Option<VisibleRelationMetadata> {
        if !self.sharing_enabled {
            self.entries.relation_reads.fetch_add(1, Ordering::Relaxed);
            return read();
        }
        read_once(
            &self.entries.relations,
            (basis, version, id),
            read,
            &self.entries.relation_reads,
            &self.entries.reuse_hits,
        )
    }

    pub(crate) fn adjacency(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        outgoing: bool,
        gather: impl FnOnce() -> Vec<RelationId>,
    ) -> Arc<[RelationId]> {
        let read = || {
            let ids: Arc<[RelationId]> = gather().into();
            self.entries
                .adjacency_relation_ids
                .fetch_add(ids.len(), Ordering::Relaxed);
            ids
        };
        if !self.sharing_enabled {
            self.entries
                .adjacency_gathers
                .fetch_add(1, Ordering::Relaxed);
            return read();
        }
        read_once(
            &self.entries.adjacency,
            (basis, version, id, outgoing),
            read,
            &self.entries.adjacency_gathers,
            &self.entries.reuse_hits,
        )
    }
}
