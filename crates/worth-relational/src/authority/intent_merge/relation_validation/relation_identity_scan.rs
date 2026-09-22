use std::collections::BTreeSet;

use crate::capabilities::StorageRead;
use crate::runtime::RuntimeInstrumentation;
use crate::transactions::data::EntityReference;

pub(super) fn existing_relation_targets_for_source(
    state: &impl StorageRead,
    instrumentation: &RuntimeInstrumentation,
    partition_id: crate::identity::data::PartitionId,
    kind_id: crate::identity::data::KindId,
    source: &EntityReference,
    targets: &BTreeSet<EntityReference>,
    excluded_relation_ids: &BTreeSet<crate::identity::data::RelationId>,
) -> bool {
    let EntityReference::Existing(source_entity) = source else {
        return false;
    };
    let Some(source_partition) = state.get_partition(source_entity.partition_id) else {
        return false;
    };
    let Some(outgoing_relations) = source_partition.adjacency.get(source_entity.slot_index())
    else {
        return false;
    };

    for relation_id in outgoing_relations.current_ids().iter().copied() {
        instrumentation.count(|counters| counters.relation_identity_candidates_scanned += 1);
        if relation_candidate_matches_target_identity(
            state,
            partition_id,
            kind_id,
            source_entity,
            targets,
            excluded_relation_ids,
            relation_id,
        ) {
            return true;
        }
    }
    false
}

fn relation_candidate_matches_target_identity(
    state: &impl StorageRead,
    partition_id: crate::identity::data::PartitionId,
    kind_id: crate::identity::data::KindId,
    source_entity: &crate::identity::data::EntityId,
    targets: &BTreeSet<EntityReference>,
    excluded_relation_ids: &BTreeSet<crate::identity::data::RelationId>,
    relation_id: crate::identity::data::RelationId,
) -> bool {
    if excluded_relation_ids.contains(&relation_id) || relation_id.partition_id != partition_id {
        return false;
    }
    let Some(relation_partition) = state.get_partition(relation_id.partition_id) else {
        return false;
    };
    let Some(relation_slot) = relation_partition.relation_arena.get(&relation_id) else {
        return false;
    };
    if relation_slot.kind_id() != Some(kind_id) {
        return false;
    }
    let Some(endpoints) = relation_slot.extra().endpoints.as_ref() else {
        return false;
    };

    endpoints.source == *source_entity
        && targets.contains(&EntityReference::Existing(endpoints.target))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
    use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
    use crate::runtime::RuntimeInstrumentation;
    use crate::storage::overlay::{PartitionState, WorkingState};
    use crate::storage::partition::AdjacencySet;
    use crate::storage::substrate::{EntityArena, RelationArena, RelationEndpoints, RelationExtra};
    use crate::storage::substrate::{EntityRecordKind, RecordKind, SlotInit};
    use crate::transactions::data::EntityReference;

    use super::existing_relation_targets_for_source;

    #[test]
    fn existing_relation_targets_scans_shared_source_once_for_batched_targets() {
        let adjacency_policy = AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        };
        let partition_id = PartitionId(1);
        let mut entity_arena = EntityArena::with_capacity(3);
        let (source_slot, source_generation, _) = entity_arena.push_slot(SlotInit {
            partition_id,
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: EntityRecordKind::empty_extra(),
        });
        let (left_slot, left_generation, _) = entity_arena.push_slot(SlotInit {
            partition_id,
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: EntityRecordKind::empty_extra(),
        });
        let (right_slot, right_generation, _) = entity_arena.push_slot(SlotInit {
            partition_id,
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: EntityRecordKind::empty_extra(),
        });
        let source = EntityId::new(partition_id, source_slot as u64, source_generation);
        let left = EntityId::new(partition_id, left_slot as u64, left_generation);
        let right = EntityId::new(partition_id, right_slot as u64, right_generation);

        let mut relation_arena = RelationArena::with_capacity(1);
        let (relation_slot, relation_generation, _) = relation_arena.push_slot(SlotInit {
            partition_id,
            kind_id: KindId(9),
            version_id: VersionId(1),
            extra: RelationExtra {
                endpoints: Some(RelationEndpoints {
                    source,
                    target: left,
                }),
                authoritative_aspect_state: None,
            },
        });
        let relation_id = RelationId::new(partition_id, relation_slot as u64, relation_generation);
        let mut adjacency = vec![AdjacencySet::new(&adjacency_policy); 3];
        adjacency[source_slot].insert(KindId(9), relation_id);

        let mut partitions = BTreeMap::new();
        partitions.insert(
            partition_id,
            PartitionState {
                partition_id,
                adjacency_policy,
                relation_overlay_is_sparse: false,
                entity_arena,
                relation_arena,
                adjacency: adjacency.into(),
                reverse_adjacency: vec![
                    AdjacencySet::new(&AdjacencyPolicy {
                        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
                        small_degree_inline_capacity: 4,
                    });
                    3
                ]
                .into(),
            },
        );
        let state = WorkingState::new(
            partitions,
            AdjacencyPolicy {
                backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
                small_degree_inline_capacity: 4,
            },
        );
        let instrumentation = RuntimeInstrumentation::new();
        let found = existing_relation_targets_for_source(
            &state,
            &instrumentation,
            partition_id,
            KindId(9),
            &EntityReference::Existing(source),
            &BTreeSet::from([
                EntityReference::Existing(left),
                EntityReference::Existing(right),
            ]),
            &BTreeSet::new(),
        );
        let counters = instrumentation
            .complexity_counters
            .lock()
            .expect("complexity counter lock poisoned")
            .clone();

        assert!(found);
        assert_eq!(counters.relation_identity_candidates_scanned, 1);
    }
}
