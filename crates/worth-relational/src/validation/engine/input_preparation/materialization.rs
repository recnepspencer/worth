use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::validation::engine::state_view::slot_resolution::AspectLocation;
use crate::validation::engine::state_view::{VisibleEntityMetadata, VisibleRelationMetadata};

use super::cache_map::CandidateInputMap;
use super::plan::{CandidateInputBasis, SharedCandidateInputs};

impl SharedCandidateInputs {
    pub(crate) fn temporary_allocation(&self) -> super::allocation::CandidateInputTemporary<'_> {
        self.allocation.temporary()
    }
    // The map lock ends before initialization. A failed initializer remains a
    // typed request refusal, distinct from a successfully cached absent value.
    fn read_once<K: Ord + Copy, V: Clone>(
        &self,
        map: &CandidateInputMap<K, V>,
        key: K,
        read: impl FnOnce() -> Option<V>,
        physical_reads: &AtomicUsize,
    ) -> Option<V> {
        if !self.sharing_enabled && !self.allocation.is_leased() {
            physical_reads.fetch_add(1, Ordering::Relaxed);
            return read();
        }
        let cell = map.cell(key, &self.allocation)?;
        let mut initialized = false;
        let value = cell
            .get_or_init(|| {
                initialized = true;
                physical_reads.fetch_add(1, Ordering::Relaxed);
                read()
            })
            .clone();
        if !initialized {
            self.entries.reuse_hits.fetch_add(1, Ordering::Relaxed);
        }
        value
    }

    pub(crate) fn gather_slice<T>(
        &self,
        count: usize,
        fill: impl FnOnce(&mut Vec<T>) -> bool,
    ) -> Option<Arc<[T]>> {
        if !self.allocation.claim_array::<T>(count) {
            return None;
        }
        let mut values = Vec::with_capacity(count);
        let filled = fill(&mut values);
        assert!(
            values.len() <= count,
            "borrowed input count bounds materialization"
        );
        let vector_bytes = (values.capacity() * std::mem::size_of::<T>()) as u64;
        if !filled || self.allocation.failure().is_some() {
            drop(values);
            self.allocation.release(vector_bytes);
            return None;
        }
        if !self.allocation.claim_arc_slice::<T>(values.len()) {
            drop(values);
            self.allocation.release(vector_bytes);
            return None;
        }
        let values = Arc::from(values);
        self.allocation.release(vector_bytes);
        Some(values)
    }

    pub(crate) fn with_temporary_vec<T, R>(
        &self,
        count: usize,
        operation: impl FnOnce(&mut Vec<T>) -> R,
    ) -> Option<R> {
        if !self.allocation.claim_array::<T>(count) {
            return None;
        }
        let mut values = Vec::with_capacity(count);
        let result = operation(&mut values);
        let bytes = (values.capacity() * std::mem::size_of::<T>()) as u64;
        drop(values);
        self.allocation.release(bytes);
        Some(result)
    }

    pub(crate) fn touched_partitions(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        count: impl FnOnce() -> Option<Option<usize>>,
        fill: impl FnOnce(&mut Vec<PartitionId>) -> bool,
    ) -> Option<Arc<[PartitionId]>> {
        self.read_once(
            &self.entries.touched_partitions,
            (basis, version),
            || match count()? {
                Some(count) => self.gather_slice(count, fill).map(Some),
                None => Some(None),
            },
            &self.entries.touched_partition_gathers,
        )
        .flatten()
    }

    pub(crate) fn touched_entity_slots(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        partition: PartitionId,
        count: impl FnOnce() -> Option<Option<usize>>,
        fill: impl FnOnce(&mut Vec<usize>) -> bool,
    ) -> Option<Arc<[usize]>> {
        self.read_once(
            &self.entries.touched_entity_slots,
            (basis, version, partition),
            || match count()? {
                Some(count) => self.gather_slice(count, fill).map(Some),
                None => Some(None),
            },
            &self.entries.touched_entity_slot_gathers,
        )
        .flatten()
    }

    pub(crate) fn touched_relation_slots(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        partition: PartitionId,
        count: impl FnOnce() -> Option<Option<usize>>,
        fill: impl FnOnce(&mut Vec<usize>) -> bool,
    ) -> Option<Arc<[usize]>> {
        self.read_once(
            &self.entries.touched_relation_slots,
            (basis, version, partition),
            || match count()? {
                Some(count) => self.gather_slice(count, fill).map(Some),
                None => Some(None),
            },
            &self.entries.touched_relation_slot_gathers,
        )
        .flatten()
    }

    pub(crate) fn touched_entities(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        count: usize,
        fill: impl FnOnce(&mut Vec<EntityId>) -> bool,
    ) -> Option<Arc<[EntityId]>> {
        self.read_once(
            &self.entries.touched_entities,
            (basis, version),
            || self.gather_slice(count, fill),
            &self.entries.touched_entity_gathers,
        )
    }

    pub(crate) fn touched_relations(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        count: usize,
        fill: impl FnOnce(&mut Vec<RelationId>) -> bool,
    ) -> Option<Arc<[RelationId]>> {
        self.read_once(
            &self.entries.touched_relations,
            (basis, version),
            || self.gather_slice(count, fill),
            &self.entries.touched_relation_gathers,
        )
    }

    pub(crate) fn entity_aspect(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        read: impl FnOnce() -> Option<AspectLocation>,
    ) -> Option<AspectLocation> {
        self.read_once(
            &self.entries.entity_aspects,
            (basis, version, id),
            || Some(read()),
            &self.entries.entity_aspect_reads,
        )
        .flatten()
    }

    pub(crate) fn relation_aspect(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: RelationId,
        read: impl FnOnce() -> Option<AspectLocation>,
    ) -> Option<AspectLocation> {
        self.read_once(
            &self.entries.relation_aspects,
            (basis, version, id),
            || Some(read()),
            &self.entries.relation_aspect_reads,
        )
        .flatten()
    }

    pub(crate) fn entity(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        read: impl FnOnce() -> Option<VisibleEntityMetadata>,
    ) -> Option<VisibleEntityMetadata> {
        self.read_once(
            &self.entries.entities,
            (basis, version, id),
            || Some(read()),
            &self.entries.entity_reads,
        )
        .flatten()
    }

    pub(crate) fn relation(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: RelationId,
        read: impl FnOnce() -> Option<VisibleRelationMetadata>,
    ) -> Option<VisibleRelationMetadata> {
        self.read_once(
            &self.entries.relations,
            (basis, version, id),
            || Some(read()),
            &self.entries.relation_reads,
        )
        .flatten()
    }

    pub(crate) fn adjacency_count(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        outgoing: bool,
        read: impl FnOnce() -> usize,
    ) -> usize {
        self.read_once(
            &self.entries.adjacency_counts,
            (basis, version, id, outgoing),
            || Some(read()),
            &self.entries.adjacency_count_reads,
        )
        .unwrap_or(0)
    }

    pub(crate) fn adjacency(
        &self,
        basis: CandidateInputBasis,
        version: VersionId,
        id: EntityId,
        outgoing: bool,
        count: usize,
        fill: impl FnOnce(&mut Vec<RelationId>) -> bool,
    ) -> Option<Arc<[RelationId]>> {
        self.read_once(
            &self.entries.adjacency,
            (basis, version, id, outgoing),
            || {
                let ids = self.gather_slice(count, fill)?;
                self.entries
                    .adjacency_relation_ids
                    .fetch_add(ids.len(), Ordering::Relaxed);
                Some(ids)
            },
            &self.entries.adjacency_gathers,
        )
    }
}
