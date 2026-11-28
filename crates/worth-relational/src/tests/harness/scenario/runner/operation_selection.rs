use crate::facade::history::BranchId;
use crate::facade::identity::{EntityId, PartitionId};
use crate::facade::snapshots::SnapshotHandle;

use super::{ActiveRelation, SeededScenarioConfig};
use crate::tests::harness::scenario::operation::ScenarioOperation;
use crate::tests::harness::scenario::seed::DeterministicGenerator;

pub(super) fn choose_operation(
    generator: &mut DeterministicGenerator,
    entities: &[EntityId],
    snapshots: &[SnapshotHandle],
    relations: &[ActiveRelation],
    branches: &[BranchId],
    config: &SeededScenarioConfig,
) -> ScenarioOperation {
    match generator.next_u64() % 15 {
        0 => ScenarioOperation::CreateEntity {
            partition: scenario_partition(generator.next_u64()),
            name: String::new(),
        },
        1 | 2 => ScenarioOperation::UpdateEntity {
            entity_slot: random_slot(generator, entities.len()),
            name: String::new(),
            branch_slot: random_slot(generator, branches.len()),
        },
        3 if config.replacement_pressure && !entities.is_empty() => {
            ScenarioOperation::ReplaceEntity {
                entity_slot: random_slot(generator, entities.len()),
                name: String::new(),
                branch_slot: 0,
                partition: scenario_partition(generator.next_u64()),
            }
        }
        4 if !entities.is_empty() => ScenarioOperation::DeleteEntity {
            entity_slot: random_slot(generator, entities.len()),
            branch_slot: 0,
        },
        5 if config.relation_pressure && entities.len() >= 2 => ScenarioOperation::CreateRelation {
            source_slot: random_slot(generator, entities.len()),
            target_slot: random_slot(generator, entities.len()),
            client_key: String::new(),
            partition: scenario_partition(generator.next_u64()),
        },
        6 if config.relation_pressure && !relations.is_empty() => {
            ScenarioOperation::DeleteRelation {
                relation_slot: random_slot(generator, relations.len()),
            }
        }
        7 if config.branch_pressure => ScenarioOperation::CreateBranch {
            branch_name: String::new(),
            from_branch_slot: random_slot(generator, branches.len()),
        },
        9 => ScenarioOperation::CaptureSnapshot,
        10 if !snapshots.is_empty() => ScenarioOperation::ReleaseSnapshot {
            snapshot_slot: random_slot(generator, snapshots.len()),
        },
        _ => ScenarioOperation::UpdateEntity {
            entity_slot: random_slot(generator, entities.len()),
            name: String::new(),
            branch_slot: random_slot(generator, branches.len()),
        },
    }
}

fn random_slot(generator: &mut DeterministicGenerator, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (generator.next_u64() as usize) % len
    }
}

fn scenario_partition(value: u64) -> PartitionId {
    match value % 4 {
        0 => PartitionId(7),
        1 => PartitionId(11),
        2 => PartitionId(29),
        _ => PartitionId(31),
    }
}
