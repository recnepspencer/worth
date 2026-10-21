use crate::identity::data::{EntityId, PartitionId};
use crate::transactions::data::{
    AspectFieldPatch, EntityMutationIntent, MergedCommitPlan, MutationIntent,
    RevalidateEntityIntent, TransactionId, UpdateEntityFieldsIntent,
};

use super::{
    select_entity_working_set_layout, sparse_entity_slots_for_plan, EntityWorkingSetLayout,
    PartitionCloneMode, AOSOA_ENTITY_CHUNK_WIDTH_SMALL,
};

fn update_intent(partition: u32, slot: u64) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id: EntityId::new(PartitionId(partition), slot, 1),
            fields: AspectFieldPatch::default(),
        },
    ))
}

fn revalidation_intent(partition: u32, slot: u64) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::Revalidate(RevalidateEntityIntent {
        entity_id: EntityId::new(PartitionId(partition), slot, 1),
    }))
}

fn merged_plan(intents: Vec<MutationIntent>) -> MergedCommitPlan {
    MergedCommitPlan {
        transaction_id: TransactionId(1),
        merged_intents: intents,
    }
}

#[test]
fn sparse_entity_slots_accepts_multi_partition_update_batches() {
    let plan = merged_plan(vec![
        update_intent(1, 0),
        update_intent(1, 1),
        update_intent(2, 7),
        update_intent(3, 9),
    ]);

    let sparse = sparse_entity_slots_for_plan(PartitionCloneMode::EntityOnly, &plan, None)
        .expect("multi-partition entity batch should stay on sparse path");

    assert_eq!(sparse.len(), 3);
    assert_eq!(
        sparse.get(&PartitionId(1)).map(|slots| slots.len()),
        Some(2)
    );
    assert_eq!(
        sparse.get(&PartitionId(2)).map(|slots| slots.len()),
        Some(1)
    );
    assert_eq!(
        sparse.get(&PartitionId(3)).map(|slots| slots.len()),
        Some(1)
    );
}

#[test]
fn sparse_layout_selection_uses_sparse_slot_count_not_full_partition_clone_width() {
    let layout = select_entity_working_set_layout(PartitionCloneMode::EntityOnly, 96);
    assert_eq!(
        layout,
        EntityWorkingSetLayout::AoSoACandidate {
            chunk_width: AOSOA_ENTITY_CHUNK_WIDTH_SMALL,
        }
    );
}

#[test]
fn sparse_entity_slots_keep_updates_and_revalidation_demands_on_the_narrow_path() {
    let plan = merged_plan(vec![update_intent(1, 3), revalidation_intent(1, 11)]);

    let sparse = sparse_entity_slots_for_plan(PartitionCloneMode::EntityOnly, &plan, None)
        .expect("an update plus an unchanged-record demand remains sparse");

    assert_eq!(
        sparse.get(&PartitionId(1)),
        Some(&std::collections::BTreeSet::from([3, 11])),
        "the sparse clone must name both the written slot and the demanded slot"
    );
}

#[test]
fn sparse_entity_selection_preserves_both_admission_limits() {
    for (partitions, slots, admitted) in [
        (8, 1, true),
        (9, 1, false),
        (1, 1024, true),
        (1, 1025, false),
    ] {
        let plan = merged_plan(
            (1..=partitions)
                .flat_map(|partition| (0..slots).map(move |slot| update_intent(partition, slot)))
                .collect(),
        );
        assert_eq!(
            sparse_entity_slots_for_plan(PartitionCloneMode::GraphSparseEntities, &plan, None)
                .is_some(),
            admitted
        );
    }
}

#[test]
fn supported_creation_only_graph_has_an_admitted_empty_entity_selection() {
    use crate::transactions::data::{CreateIntent, EntitySpec};
    let plan = merged_plan(vec![MutationIntent::Create(CreateIntent::Entity(
        EntitySpec {
            partition_id: PartitionId(1),
            kind_id: crate::identity::data::KindId(1),
            client_key: crate::symbols::data::ClientKey::raw("created"),
            fields: AspectFieldPatch::default(),
        },
    ))]);
    assert_eq!(
        sparse_entity_slots_for_plan(PartitionCloneMode::GraphSparseEntities, &plan, None),
        Some(Default::default())
    );
}
