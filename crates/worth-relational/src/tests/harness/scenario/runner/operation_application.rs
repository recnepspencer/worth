use crate::facade::history::BranchId;
use crate::facade::identity::EntityId;
use crate::facade::runtime::RelationalRuntime;
use crate::facade::snapshots::SnapshotHandle;
use crate::facade::transactions::{
    DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent, MutationIntent,
    RelationMutationIntent, ReplaceEntityIntent, WorkerIntentBatch,
};
use crate::tests::harness::scenario::operation::ScenarioOperation;
use crate::tests::harness::scenario::seed::DeterministicGenerator;
use crate::tests::support::{
    create_relation_in_partition, release_test_commit_snapshot, try_update_entity_on_branch, KindId,
};

use super::{scenario_branch_main, ActiveRelation};

pub(super) fn apply_operation(
    runtime: &RelationalRuntime,
    entities: &mut Vec<EntityId>,
    snapshots: &mut Vec<SnapshotHandle>,
    relations: &mut Vec<ActiveRelation>,
    branches: &mut Vec<BranchId>,
    generator: &mut DeterministicGenerator,
    name_counter: &mut u64,
    seed: u64,
    step: usize,
    operation: &ScenarioOperation,
) {
    match operation {
        ScenarioOperation::CreateEntity { partition, .. } => {
            let name = format!("seed-{seed}-create-{step}-{name_counter}");
            *name_counter += 1;
            let entity =
                crate::tests::support::create_entity_in_partition(runtime, &name, *partition);
            entities.push(entity);
        }
        ScenarioOperation::UpdateEntity {
            entity_slot,
            branch_slot,
            ..
        } => {
            if entities.is_empty() {
                return;
            }
            let index = (*entity_slot).min(entities.len() - 1);
            let branch = branches[(*branch_slot).min(branches.len() - 1)].clone();
            let name = format!("seed-{seed}-update-{step}-{name_counter}");
            *name_counter += 1;
            if let Ok(outcome) =
                try_update_entity_on_branch(runtime, entities[index], &name, branch)
            {
                release_test_commit_snapshot(runtime, &outcome);
            }
        }
        ScenarioOperation::ReplaceEntity {
            entity_slot,
            branch_slot,
            partition,
            ..
        } => {
            replace_entity_through_authoritative_patch(
                runtime,
                entities,
                relations,
                *entity_slot,
                *branch_slot,
                *partition,
                name_counter,
                seed,
                step,
                branches,
            );
        }
        ScenarioOperation::CreateRelation {
            source_slot,
            target_slot,
            partition,
            ..
        } => {
            create_live_relation(
                runtime,
                entities,
                relations,
                generator,
                *source_slot,
                *target_slot,
                *partition,
                name_counter,
                seed,
                step,
            );
        }
        ScenarioOperation::CaptureSnapshot => {
            snapshots.push(runtime.visibility_authority().snapshot())
        }
        ScenarioOperation::ReleaseSnapshot { snapshot_slot } => {
            release_snapshot(runtime, snapshots, *snapshot_slot);
        }
        ScenarioOperation::DeleteRelation { relation_slot } => {
            delete_relation(runtime, entities, relations, *relation_slot);
        }
        ScenarioOperation::DeleteEntity {
            entity_slot,
            branch_slot,
        } => {
            delete_entity(
                runtime,
                entities,
                relations,
                branches,
                *entity_slot,
                *branch_slot,
            );
        }
        ScenarioOperation::CreateBranch {
            branch_name,
            from_branch_slot,
        } => {
            create_branch(
                runtime,
                branches,
                branch_name,
                *from_branch_slot,
                name_counter,
                seed,
                step,
            );
        }
        ScenarioOperation::MergeBranchIntoMain { branch_slot } => {
            merge_branch_into_main(runtime, entities, relations, branches, *branch_slot);
        }
        ScenarioOperation::RunRetentionPass => {
            let _ = runtime.retention().run_pass();
            refresh_live_world(runtime, entities, relations);
        }
        ScenarioOperation::DurableCheckpoint | ScenarioOperation::CompactDurableStore => {}
    }
}

fn replace_entity_through_authoritative_patch(
    runtime: &RelationalRuntime,
    entities: &mut Vec<EntityId>,
    relations: &mut Vec<ActiveRelation>,
    entity_slot: usize,
    branch_slot: usize,
    partition: crate::facade::identity::PartitionId,
    name_counter: &mut u64,
    seed: u64,
    step: usize,
    branches: &[BranchId],
) {
    if entities.is_empty() {
        return;
    }
    let index = entity_slot.min(entities.len() - 1);
    let branch = branches[branch_slot.min(branches.len() - 1)].clone();
    let name = format!("seed-{seed}-replace-{step}-{name_counter}");
    *name_counter += 1;
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch);
    txn.push_batch(
        WorkerIntentBatch::new("replace").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: entities[index],
                replacement: crate::transactions::data::EntitySpec {
                    partition_id: partition,
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw(format!(
                        "replace-{seed}-{step}-{name_counter}"
                    )),
                    fields: crate::tests::support::aspect_field_patch_from_values([(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        crate::tests::support::string_aspect_value(&name),
                    )]),
                },
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    if let Ok(outcome) = txn.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ) {
        if let Some(replacement) = crate::tests::support::changed_entities(&outcome).last() {
            entities[index] = *replacement;
        }
        release_test_commit_snapshot(runtime, &outcome);
        refresh_live_world(runtime, entities, relations);
    }
}

fn create_live_relation(
    runtime: &RelationalRuntime,
    entities: &[EntityId],
    relations: &mut Vec<ActiveRelation>,
    generator: &mut DeterministicGenerator,
    source_slot: usize,
    target_slot: usize,
    partition: crate::facade::identity::PartitionId,
    name_counter: &mut u64,
    seed: u64,
    step: usize,
) {
    if entities.len() < 2 {
        return;
    }
    let source = entities[source_slot.min(entities.len() - 1)];
    let mut target = entities[target_slot.min(entities.len() - 1)];
    if target == source {
        target = entities[(generator.next_u64() as usize) % entities.len()];
    }
    if source == target {
        return;
    }
    if relations.iter().any(|relation| {
        relation.source == source && relation.target == target && relation.partition == partition
    }) {
        return;
    }
    let client_key = format!("seed-{seed}-rel-{step}-{name_counter}");
    *name_counter += 1;
    let relation = create_relation_in_partition(runtime, source, target, &client_key, partition);
    relations.push(ActiveRelation {
        relation_id: relation,
        source,
        target,
        partition,
    });
}

fn release_snapshot(
    runtime: &RelationalRuntime,
    snapshots: &mut Vec<SnapshotHandle>,
    snapshot_slot: usize,
) {
    if snapshots.is_empty() {
        return;
    }
    let index = snapshot_slot.min(snapshots.len() - 1);
    let snapshot = snapshots.swap_remove(index);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&snapshot)
        .is_ok());
}

fn delete_relation(
    runtime: &RelationalRuntime,
    entities: &mut Vec<EntityId>,
    relations: &mut Vec<ActiveRelation>,
    relation_slot: usize,
) {
    if relations.is_empty() {
        return;
    }
    let index = relation_slot.min(relations.len() - 1);
    let relation = relations.swap_remove(index);
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: relation.relation_id,
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    release_test_commit_snapshot(runtime, &outcome);
    refresh_live_world(runtime, entities, relations);
}

fn delete_entity(
    runtime: &RelationalRuntime,
    entities: &mut Vec<EntityId>,
    relations: &mut Vec<ActiveRelation>,
    branches: &[BranchId],
    entity_slot: usize,
    branch_slot: usize,
) {
    if entities.is_empty() {
        return;
    }
    let index = entity_slot.min(entities.len() - 1);
    let deleted = entities[index];
    let branch = branches[branch_slot.min(branches.len() - 1)].clone();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch);
    txn.push_batch(
        WorkerIntentBatch::new("delete-entity").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: deleted }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    if let Ok(outcome) = txn.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ) {
        release_test_commit_snapshot(runtime, &outcome);
        entities.swap_remove(index);
        relations.retain(|relation| relation.source != deleted && relation.target != deleted);
        refresh_live_world(runtime, entities, relations);
    }
}

fn create_branch(
    runtime: &RelationalRuntime,
    branches: &mut Vec<BranchId>,
    branch_name: &str,
    from_branch_slot: usize,
    name_counter: &mut u64,
    seed: u64,
    step: usize,
) {
    let from_branch = branches[from_branch_slot.min(branches.len() - 1)].clone();
    let branch_name = if branch_name.is_empty() {
        format!("branch-{seed}-{step}-{name_counter}")
    } else {
        branch_name.to_string()
    };
    *name_counter += 1;
    let branch = BranchId(branch_name);
    if runtime
        .history_authority()
        .fork_branch_from(branch.clone(), &from_branch)
        .is_ok()
    {
        branches.push(branch);
    }
}

fn merge_branch_into_main(
    runtime: &RelationalRuntime,
    entities: &mut Vec<EntityId>,
    relations: &mut Vec<ActiveRelation>,
    branches: &[BranchId],
    branch_slot: usize,
) {
    if branches.len() <= 1 {
        return;
    }
    let branch = branches[branch_slot.min(branches.len() - 1)].clone();
    if branch.0 != "main" {
        let txn = crate::tests::support::test_owner_begin_merge_transaction(
            runtime,
            scenario_branch_main(),
            vec![branch],
        );
        if let Ok(outcome) = txn.commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ) {
            release_test_commit_snapshot(runtime, &outcome);
            refresh_live_world(runtime, entities, relations);
        }
    }
}

fn refresh_live_world(
    runtime: &RelationalRuntime,
    entities: &mut Vec<EntityId>,
    relations: &mut Vec<ActiveRelation>,
) {
    let snapshot = runtime.visibility_authority().snapshot();
    let read = runtime.read_truth().read_snapshot(&snapshot).unwrap();
    *entities = read
        .entities()
        .iter()
        .map(|record| record.entity_id)
        .collect();
    *relations = read
        .relations()
        .iter()
        .map(|record| ActiveRelation {
            relation_id: record.relation_id,
            source: record.source,
            target: record.target,
            partition: record.relation_id.partition_id,
        })
        .collect();
    drop(read);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&snapshot)
        .is_ok());
}
