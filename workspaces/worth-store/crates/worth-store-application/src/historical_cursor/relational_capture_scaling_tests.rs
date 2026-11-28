use std::collections::BTreeMap;

use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::{EntityId, PartitionId, VersionId};
use worth_relational::facade::publication::{PublishedAuthoritativePatch, RecordStructuralChange};

use super::{replay_dag_histories, AncestorRecordCommit, TopologicalCommitPatches};

fn empty_entry(cid: u64, change: RecordStructuralChange) -> AncestorRecordCommit {
    AncestorRecordCommit {
        commit_id: CommitId(cid),
        version_id: VersionId(cid),
        structural_change: change,
        changed_aspects: Vec::new(),
        patch: PublishedAuthoritativePatch::empty(),
        relation_endpoints: None,
    }
}

#[test]
fn replay_entity_history_scales_subquadratically_on_linear_500_commit_history() {
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);
    let commit_count = 500u64;

    // 500 surviving entities, each created at its own commit i (1 patch each).
    let single_commits: Vec<TopologicalCommitPatches> = (1..=commit_count)
        .map(|i| TopologicalCommitPatches {
            primary_parent: if i == 1 { None } else { Some((i - 2) as usize) },
            entities: vec![(
                EntityId::new(PartitionId::main(), i, 1),
                empty_entry(i, RecordStructuralChange::Created),
            )],
            relations: vec![],
        })
        .collect();
    let single = replay_dag_histories(
        &exec,
        &single_commits,
        CommitId(commit_count),
        VersionId(commit_count),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(single.entities.len(), commit_count as usize);
    assert!(
        single.steps <= 10_000,
        "500 single-patch entities must scale O(N log N), got {} steps",
        single.steps
    );

    // 250 entities each created at commit i and updated 250 commits later at i + 250.
    let updated_commits: Vec<TopologicalCommitPatches> = (1..=commit_count)
        .map(|i| {
            let (slot, change) = if i <= 250 {
                (i, RecordStructuralChange::Created)
            } else {
                (i - 250, RecordStructuralChange::Updated)
            };
            TopologicalCommitPatches {
                primary_parent: if i == 1 { None } else { Some((i - 2) as usize) },
                entities: vec![(
                    EntityId::new(PartitionId::main(), slot, 1),
                    empty_entry(i, change),
                )],
                relations: vec![],
            }
        })
        .collect();
    let updated = replay_dag_histories(
        &exec,
        &updated_commits,
        CommitId(commit_count),
        VersionId(commit_count),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(updated.entities.len(), 250);
    assert!(updated.entities.values().all(|frames| frames.len() == 2));
    assert!(
        updated.steps <= 12_000,
        "250 two-patch entities across 500 commits must scale O(N log N), got {} steps",
        updated.steps
    );
}

#[test]
fn replay_scales_subquadratically_on_merge_heavy_history_with_unrelated_merges() {
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);
    let record_count = 200u64;
    let merge_count = 200u64;

    let mut commits = Vec::new();
    let mut next_cid = 1u64;

    // 1. Create N = 200 records in an initial seed commit.
    let seed_cid = CommitId(next_cid);
    next_cid += 1;
    let seed_entities: Vec<(EntityId, AncestorRecordCommit)> = (1..=record_count)
        .map(|slot| {
            (
                EntityId::new(PartitionId::main(), slot, 1),
                empty_entry(seed_cid.0, RecordStructuralChange::Created),
            )
        })
        .collect();
    commits.push(TopologicalCommitPatches {
        primary_parent: None,
        entities: seed_entities,
        relations: vec![],
    });

    // 2. Perform M = 200 unrelated diamond merges (each touching distinct records).
    let mut tip_idx = 0usize;
    for m in 0..merge_count {
        let left_cid = CommitId(next_cid);
        next_cid += 1;
        let right_cid = CommitId(next_cid);
        next_cid += 1;
        let _merge_cid = CommitId(next_cid);
        next_cid += 1;

        let left_ent = EntityId::new(PartitionId::main(), 10_000 + m * 2, 1);
        let right_ent = EntityId::new(PartitionId::main(), 10_000 + m * 2 + 1, 1);
        let left_idx = commits.len();
        commits.push(TopologicalCommitPatches {
            primary_parent: Some(tip_idx),
            entities: vec![(
                left_ent,
                empty_entry(left_cid.0, RecordStructuralChange::Created),
            )],
            relations: vec![],
        });
        commits.push(TopologicalCommitPatches {
            primary_parent: Some(tip_idx),
            entities: vec![(
                right_ent,
                empty_entry(right_cid.0, RecordStructuralChange::Created),
            )],
            relations: vec![],
        });
        let merge_idx = commits.len();
        commits.push(TopologicalCommitPatches {
            primary_parent: Some(left_idx),
            entities: vec![],
            relations: vec![],
        });
        tip_idx = merge_idx;
    }

    // 3. Update all N = 200 initial records together after the M = 200 merges.
    let final_cid = CommitId(next_cid);
    let final_updates: Vec<(EntityId, AncestorRecordCommit)> = (1..=record_count)
        .map(|slot| {
            (
                EntityId::new(PartitionId::main(), slot, 1),
                empty_entry(final_cid.0, RecordStructuralChange::Updated),
            )
        })
        .collect();
    commits.push(TopologicalCommitPatches {
        primary_parent: Some(tip_idx),
        entities: final_updates,
        relations: vec![],
    });

    let replayed = replay_dag_histories(
        &exec,
        &commits,
        final_cid,
        VersionId(final_cid.0),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    for slot in 1..=record_count {
        let eid = EntityId::new(PartitionId::main(), slot, 1);
        assert_eq!(
            replayed.entities[&eid].len(),
            2,
            "each initial record must have exactly 2 frames (create + final update)"
        );
    }
    // Quadratic N * M would cross 200 * 200 = 40_000+ merge heads; persistent
    // Arc::ptr_eq subtree sharing must complete in O((N + M) log(N + M)).
    assert!(
        replayed.steps < 20_000,
        "merge-heavy replay must be O((N + M) log N), not O(N * M); got {} steps",
        replayed.steps
    );
}
