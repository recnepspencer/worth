use crate::identity::data::PartitionId;
use crate::runtime::PartitionAccess;
use crate::transactions::data::{
    EntityMutationIntent, ExistingRecordTarget, MergedCommitPlan, MutationIntent,
};

pub(super) fn touched_partitions_for_plan_set(
    current_state: &impl PartitionAccess,
    plan: &MergedCommitPlan,
) -> std::collections::BTreeSet<PartitionId> {
    let mut touched = std::collections::BTreeSet::new();
    for intent in &plan.merged_intents {
        intent.seed_touched_partitions(&mut touched);
        let widen_entity_adjacency_scope = matches!(
            intent,
            MutationIntent::Entity(EntityMutationIntent::Delete(_))
                | MutationIntent::Entity(EntityMutationIntent::Replace(_))
        );
        if let Some(target) = intent.existing_record_target() {
            match target {
                ExistingRecordTarget::Entity(entity_id) => {
                    if !widen_entity_adjacency_scope {
                        continue;
                    }
                    if let Some(partition) = current_state.get_partition(entity_id.partition_id) {
                        let slot = entity_id.slot_index();
                        if let Some(adjacency) = partition.adjacency.get(slot) {
                            for relation_id in adjacency.current_ids() {
                                include_relation_scope(current_state, &mut touched, *relation_id);
                            }
                        }
                        if let Some(adjacency) = partition.reverse_adjacency.get(slot) {
                            for relation_id in adjacency.current_ids() {
                                include_relation_scope(current_state, &mut touched, *relation_id);
                            }
                        }
                    }
                }
                ExistingRecordTarget::Relation(relation_id) => {
                    include_relation_scope(current_state, &mut touched, relation_id);
                }
            }
        }
    }
    touched
}

pub(super) fn touched_partitions_for_flat_plan_set(
    plan: &MergedCommitPlan,
) -> std::collections::BTreeSet<PartitionId> {
    let mut touched = std::collections::BTreeSet::new();
    for intent in &plan.merged_intents {
        intent.seed_touched_partitions(&mut touched);
    }
    touched
}

fn include_relation_scope(
    current_state: &impl PartitionAccess,
    touched: &mut std::collections::BTreeSet<PartitionId>,
    relation_id: crate::identity::data::RelationId,
) {
    touched.insert(relation_id.partition_id);
    if let Some(partition) = current_state.get_partition(relation_id.partition_id) {
        if let Some(endpoints) = partition
            .relation_arena
            .extra
            .get(relation_id.slot_index())
            .and_then(|extra| extra.endpoints.as_ref())
        {
            touched.insert(endpoints.source.partition_id);
            touched.insert(endpoints.target.partition_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
    use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
    use crate::storage::overlay::{PartitionState, WorkingState};
    use crate::storage::partition::AdjacencySet;
    use crate::storage::substrate::{EntityArena, RelationArena, RelationEndpoints, RelationExtra};
    use crate::transactions::data::{
        AspectFieldPatch, DeleteEntityIntent, EntityMutationIntent, MergedCommitPlan,
        MutationIntent, TransactionId, UpdateEntityFieldsIntent,
    };

    use super::touched_partitions_for_plan_set;

    #[test]
    fn delete_entity_touches_relation_and_opposite_endpoint_partitions() {
        let adjacency_policy = AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        };
        let source_partition_id = PartitionId(1);
        let relation_partition_id = PartitionId(2);
        let target_partition_id = PartitionId(3);
        let source_entity_id = EntityId::new(source_partition_id, 0, 1);
        let target_entity_id = EntityId::new(target_partition_id, 0, 1);

        let mut relation_arena = RelationArena::with_capacity(1);
        let (slot, generation, _) = relation_arena.push_slot(crate::storage::substrate::SlotInit {
            partition_id: relation_partition_id,
            kind_id: KindId(9),
            version_id: VersionId(1),
            extra: RelationExtra {
                endpoints: Some(RelationEndpoints {
                    source: source_entity_id,
                    target: target_entity_id,
                }),
                authoritative_aspect_state: None,
            },
        });
        let relation_id = RelationId::new(relation_partition_id, slot as u64, generation);

        let mut source_adjacency = AdjacencySet::new(&adjacency_policy);
        source_adjacency.insert(KindId(9), relation_id);

        let mut partitions = BTreeMap::new();
        partitions.insert(
            source_partition_id,
            PartitionState {
                partition_id: source_partition_id,
                adjacency_policy: adjacency_policy.clone(),
                relation_overlay_is_sparse: false,
                entity_arena: EntityArena::with_capacity(1),
                relation_arena: RelationArena::with_capacity(0),
                adjacency: vec![source_adjacency].into(),
                reverse_adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
            },
        );
        partitions.insert(
            relation_partition_id,
            PartitionState {
                partition_id: relation_partition_id,
                adjacency_policy: adjacency_policy.clone(),
                relation_overlay_is_sparse: false,
                entity_arena: EntityArena::with_capacity(0),
                relation_arena,
                adjacency: Default::default(),
                reverse_adjacency: Default::default(),
            },
        );
        partitions.insert(
            target_partition_id,
            PartitionState {
                partition_id: target_partition_id,
                adjacency_policy: adjacency_policy.clone(),
                relation_overlay_is_sparse: false,
                entity_arena: EntityArena::with_capacity(1),
                relation_arena: RelationArena::with_capacity(0),
                adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
                reverse_adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
            },
        );

        let state = WorkingState::new(partitions, adjacency_policy);
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(1),
            merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent {
                    entity_id: source_entity_id,
                },
            ))],
        };

        let touched = touched_partitions_for_plan_set(&state, &plan);

        assert!(touched.contains(&source_partition_id));
        assert!(touched.contains(&relation_partition_id));
        assert!(touched.contains(&target_partition_id));
    }

    #[test]
    fn update_entity_fields_does_not_widen_scope_through_existing_adjacency() {
        let adjacency_policy = AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        };
        let source_partition_id = PartitionId(1);
        let relation_partition_id = PartitionId(2);
        let target_partition_id = PartitionId(3);
        let source_entity_id = EntityId::new(source_partition_id, 0, 1);
        let target_entity_id = EntityId::new(target_partition_id, 0, 1);

        let mut relation_arena = RelationArena::with_capacity(1);
        let (slot, generation, _) = relation_arena.push_slot(crate::storage::substrate::SlotInit {
            partition_id: relation_partition_id,
            kind_id: KindId(9),
            version_id: VersionId(1),
            extra: RelationExtra {
                endpoints: Some(RelationEndpoints {
                    source: source_entity_id,
                    target: target_entity_id,
                }),
                authoritative_aspect_state: None,
            },
        });
        let relation_id = RelationId::new(relation_partition_id, slot as u64, generation);

        let mut source_adjacency = AdjacencySet::new(&adjacency_policy);
        source_adjacency.insert(KindId(9), relation_id);

        let mut partitions = BTreeMap::new();
        partitions.insert(
            source_partition_id,
            PartitionState {
                partition_id: source_partition_id,
                adjacency_policy: adjacency_policy.clone(),
                relation_overlay_is_sparse: false,
                entity_arena: EntityArena::with_capacity(1),
                relation_arena: RelationArena::with_capacity(0),
                adjacency: vec![source_adjacency].into(),
                reverse_adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
            },
        );
        partitions.insert(
            relation_partition_id,
            PartitionState {
                partition_id: relation_partition_id,
                adjacency_policy: adjacency_policy.clone(),
                relation_overlay_is_sparse: false,
                entity_arena: EntityArena::with_capacity(0),
                relation_arena,
                adjacency: Default::default(),
                reverse_adjacency: Default::default(),
            },
        );
        partitions.insert(
            target_partition_id,
            PartitionState {
                partition_id: target_partition_id,
                adjacency_policy: adjacency_policy.clone(),
                relation_overlay_is_sparse: false,
                entity_arena: EntityArena::with_capacity(1),
                relation_arena: RelationArena::with_capacity(0),
                adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
                reverse_adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
            },
        );

        let state = WorkingState::new(partitions, adjacency_policy);
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(1),
            merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: source_entity_id,
                    fields: AspectFieldPatch::default(),
                },
            ))],
        };

        let touched = touched_partitions_for_plan_set(&state, &plan);

        assert!(touched.contains(&source_partition_id));
        assert!(!touched.contains(&relation_partition_id));
        assert!(!touched.contains(&target_partition_id));
    }
}
