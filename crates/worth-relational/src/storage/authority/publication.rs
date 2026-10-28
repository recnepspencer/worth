use std::collections::{BTreeMap, BTreeSet};

use crate::storage::overlay::{PartitionAccess, PartitionMutationJournal, PartitionState};

use super::StorageAuthority;

mod execution;
mod plan;
pub(crate) use plan::PreparedMirrorPublication;

/// Exact partition set published by the storage owner for one operation.
///
/// Construction stays behind `StorageAuthority`, so downstream root assembly
/// consumes carried mutation truth instead of rediscovering it from storage.
#[derive(Debug, Clone)]
pub(crate) struct RelationalPublishedPartitionDelta {
    partition_ids: BTreeSet<crate::identity::data::PartitionId>,
    /// The exact owner-produced overlays and journals.  A branch root uses
    /// these against its own prior regions; it must never rediscover a
    /// partition from the runtime-wide current map.
    publications:
        BTreeMap<crate::identity::data::PartitionId, (PartitionState, PartitionMutationJournal)>,
}

impl RelationalPublishedPartitionDelta {
    pub(crate) fn from_committed_partitions(
        committed_partitions: &BTreeMap<
            crate::identity::data::PartitionId,
            (PartitionState, PartitionMutationJournal),
        >,
    ) -> Self {
        let publications = committed_partitions
            .iter()
            .map(|(&partition_id, (partition, journal))| {
                let mut canonical_partition = partition.clone();
                canonical_partition.clear_runtime_pin_counters();
                (partition_id, (canonical_partition, journal.clone()))
            })
            .collect();
        Self {
            partition_ids: committed_partitions.keys().copied().collect(),
            publications,
        }
    }
    pub(crate) fn partition_ids(
        &self,
    ) -> impl Iterator<Item = crate::identity::data::PartitionId> + '_ {
        self.partition_ids.iter().copied()
    }

    pub(crate) fn len(&self) -> usize {
        self.partition_ids.len()
    }

    pub(crate) fn publication(
        &self,
        partition_id: crate::identity::data::PartitionId,
    ) -> Option<(&PartitionState, &PartitionMutationJournal)> {
        self.publications
            .get(&partition_id)
            .map(|(partition, journal)| (partition, journal))
    }

    pub(crate) fn projected_partitions_from_access(
        &self,
        current: &impl PartitionAccess,
    ) -> BTreeMap<crate::identity::data::PartitionId, PartitionState> {
        let mut projected = current
            .partition_ids()
            .into_iter()
            .filter_map(|partition_id| {
                current
                    .get_partition(partition_id)
                    .cloned()
                    .map(|partition| (partition_id, partition))
            })
            .collect::<BTreeMap<_, _>>();
        for (partition_id, (overlay, journal)) in &self.publications {
            let partition = if let Some(current) = current.get_partition(*partition_id) {
                let mut partition = current.clone();
                let mut overlay = overlay.clone();
                partition.merge_overlay_from_owned(&mut overlay, journal);
                partition
            } else {
                overlay.clone()
            };
            projected.insert(*partition_id, partition);
        }
        projected
    }
}

impl StorageAuthority<'_> {
    pub(crate) fn publish_prepared_mirror(&self, prepared: PreparedMirrorPublication) {
        execution::execute_partition_publication(self, prepared);
    }

    pub(crate) fn affirm_no_partition_changes(&self) -> RelationalPublishedPartitionDelta {
        RelationalPublishedPartitionDelta {
            partition_ids: BTreeSet::new(),
            publications: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RelationalPublishedPartitionDelta;
    use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
    use crate::identity::data::{EntityId, KindId, PartitionId, VersionId};
    use crate::storage::overlay::{PartitionMutationJournal, PartitionState};
    use crate::storage::substrate::{
        EntityArena, EntityRecordKind, RecordKind, RelationArena, RelationEndpoints, RelationExtra,
        SlotInit,
    };
    use std::collections::BTreeMap;

    #[test]
    fn initial_root_projection_sparse_merges_partially_existing_partition() {
        let partition_id = PartitionId(19);
        let policy = AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        };
        let mut current_partition = PartitionState {
            partition_id,
            adjacency_policy: policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena: EntityArena::with_capacity(1),
            relation_arena: RelationArena::with_capacity(8),
            adjacency: Default::default(),
            reverse_adjacency: Default::default(),
        };
        current_partition.relation_arena.push_slot(SlotInit {
            partition_id,
            kind_id: KindId(2),
            version_id: VersionId(1),
            extra: RelationExtra {
                endpoints: Some(RelationEndpoints {
                    source: EntityId::new(partition_id, 0, 1),
                    target: EntityId::new(partition_id, 1, 1),
                }),
                authoritative_aspect_state: None,
            },
        });
        let mut overlay = PartitionState {
            partition_id,
            adjacency_policy: policy,
            relation_overlay_is_sparse: false,
            entity_arena: EntityArena::with_capacity(1),
            relation_arena: RelationArena::with_capacity(0),
            adjacency: Default::default(),
            reverse_adjacency: Default::default(),
        };
        overlay.entity_arena.push_slot(SlotInit {
            partition_id,
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: EntityRecordKind::empty_extra(),
        });
        let mut journal = PartitionMutationJournal::default();
        journal.entity_slots.insert(0);
        let delta = RelationalPublishedPartitionDelta::from_committed_partitions(&BTreeMap::from(
            [(partition_id, (overlay, journal))],
        ));

        let projected = delta
            .projected_partitions_from_access(&BTreeMap::from([(partition_id, current_partition)]));
        let projected = &projected[&partition_id];
        assert_eq!(projected.entity_arena.slot_count(), 1);
        assert_eq!(projected.relation_arena.slot_count(), 1);
    }
}
