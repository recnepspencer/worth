//! Literal business edits are checked against independent full-key partitions.
use super::{
    checkpoint::persisted,
    fixtures::*,
    lifecycle,
    oracle::{Anchor, Carrier, Oracle},
};
use crate::facade::replay::{
    RelationalReplayRequest, ReplayExecutionMode, ReplayObservableSurface, ReplayVerificationMode,
};
use std::collections::BTreeSet;
use worth_foundational::facade::{AggregateIdentity, AggregateLineageKind};

pub(super) type Event = (
    crate::history::data::CommitId,
    AggregateLineageKind,
    AggregateIdentity,
    AggregateIdentity,
);
pub(super) type Group = (Vec<NativeMemberKey>, AggregateIdentity, Vec<Event>);
pub(super) fn state(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    groups: &[BTreeSet<NativeMemberKey>],
) -> Vec<Group> {
    groups
        .iter()
        .map(|group| {
            let read = read(execution, runtime, snapshot, group.first().unwrap().clone());
            (
                read.members().cloned().collect(),
                read.identity(),
                read.lineage()
                    .map(|event| (event.occurrence(), event.kind(), event.from(), event.to()))
                    .collect(),
            )
        })
        .collect()
}
fn audit(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, target: &CommitResult) {
    let audit = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: target.commit.commit_id,
                branch_id: BranchId("main".into()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert_eq!(audit.failure, None, "{:?}", audit.mismatches);
    assert!(audit.mismatches.is_empty());
    assert_eq!(audit.snapshot_version, Some(target.commit.version_id));
    assert!(audit
        .reconstructed_commit_closure
        .contains(&target.commit.commit_id));
    assert!(audit
        .compared_surfaces
        .contains(&ReplayObservableSurface::Snapshot));
}
fn next(seed: &mut u64) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed as usize
}
fn toggle(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    carrier: EntityId,
    vertex: usize,
    vertices: &[EntityId],
    oracle: &mut Oracle,
    serial: usize,
) -> WorkerIntentBatch {
    let had = oracle.carriers[&carrier].vertices.contains(&vertex);
    let label = format!("seeded-incidence-{serial}");
    if had {
        let truth = runtime
            .read_truth()
            .read_snapshot(snapshot, execution)
            .unwrap()
            .unwrap();
        let edge = truth
            .relations()
            .iter()
            .find(|edge| {
                edge.source == carrier
                    && edge.target == vertices[vertex]
                    && edge.kind.kind_id == KindId(INCIDENCE)
            })
            .expect("literal positive incidence has its own genuine physical occurrence")
            .relation_id;
        oracle
            .carriers
            .get_mut(&carrier)
            .unwrap()
            .vertices
            .remove(&vertex);
        WorkerIntentBatch::new(label).push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: edge }),
        ))
    } else {
        oracle
            .carriers
            .get_mut(&carrier)
            .unwrap()
            .vertices
            .insert(vertex);
        WorkerIntentBatch::new(label.clone()).push(edge(
            &label,
            INCIDENCE,
            EntityReference::Existing(carrier),
            EntityReference::Existing(vertices[vertex]),
        ))
    }
}
#[test]
fn seeded_full_key_differential_fork_recovery_and_replay_keep_actual_names_and_lineage() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = persisted(registry());
    let mut batch = WorkerIntentBatch::new("seeded-literal-world");
    for index in 0..8 {
        batch = batch.push(entity(1, index));
    }
    for index in 0..4 {
        batch = batch.push(entity(6, index));
    }
    for index in 0..2 {
        batch = batch.push(entity(7, index)).push(entity(8, index));
    }
    for index in 0..8 {
        batch = batch.push(edge(
            &format!("initial-vertex-{index}"),
            INCIDENCE,
            EntityReference::Created(created(1, index)),
            EntityReference::Created(created(6, index % 4)),
        ));
        for side in 0..2 {
            batch = batch.push(edge(
                &format!("initial-anchor-{index}-{side}"),
                lifecycle::anchor_kind(side, Anchor::Exterior(0)),
                EntityReference::Created(created(1, index)),
                lifecycle::anchor_reference(Anchor::Exterior(0)),
            ));
        }
    }
    let born = commit(execution, &runtime, batch);
    let mut carriers = (0..8)
        .map(|index| born.created_entity(&created(1, index)).unwrap())
        .collect::<Vec<_>>();
    let vertices = (0..4)
        .map(|index| born.created_entity(&created(6, index)).unwrap())
        .collect::<Vec<_>>();
    let mut oracle = Oracle::default();
    for (index, carrier) in carriers.iter().enumerate() {
        oracle.carriers.insert(
            *carrier,
            Carrier {
                vertices: BTreeSet::from([index % 4]),
                anchors: [Anchor::Exterior(0); 2],
            },
        );
    }
    oracle.check(execution, &runtime, &born);
    let original_groups = oracle.groups();
    let original = state(execution, &runtime, &born.snapshot, &original_groups);
    let fork = create_branch_from_main(&runtime, "logical-seeded-fork");
    let fork_snapshot = snapshot_for_owner_branch(&runtime, &fork);
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let mut current = born.snapshot.clone();
    let mut seed = 0x9181_0006_62f3_u64;
    let mut saw_split = false;
    let mut saw_merge = false;
    let mut final_result = None;
    for step in 0..64 {
        let index = if step < 4 {
            0
        } else {
            next(&mut seed) % carriers.len()
        };
        let carrier = carriers[index];
        let batch = if step == 24 {
            let retired = carriers[3];
            let truth = runtime
                .read_truth()
                .read_snapshot(&current, execution)
                .unwrap()
                .unwrap();
            let mut batch = WorkerIntentBatch::new("seeded-actual-retirement");
            for relation in truth.relations().iter().filter(|edge| {
                edge.source == retired
                    && [
                        INCIDENCE,
                        NEG_REGION,
                        POS_REGION,
                        NEG_EXTERIOR,
                        POS_EXTERIOR,
                    ]
                    .contains(&edge.kind.kind_id.as_u32())
            }) {
                batch = batch.push(MutationIntent::Relation(RelationMutationIntent::Delete(
                    DeleteRelationIntent {
                        relation_id: relation.relation_id,
                    },
                )));
            }
            oracle.carriers.remove(&retired);
            batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id: retired },
            )))
        } else if step == 25 {
            let mut batch = WorkerIntentBatch::new("seeded-actual-birth").push(entity(1, 8));
            for side in 0..2 {
                batch = batch.push(edge(
                    &format!("new-anchor-{side}"),
                    lifecycle::anchor_kind(side, Anchor::Region(1)),
                    EntityReference::Created(created(1, 8)),
                    EntityReference::Existing(born.created_entity(&created(7, 1)).unwrap()),
                ));
            }
            batch
        } else if step < 2 || (step >= 4 && step % 2 == 0) {
            let vertex = if step < 2 { 0 } else { next(&mut seed) % 4 };
            toggle(
                execution,
                &runtime,
                &current,
                carrier,
                vertex,
                &vertices,
                &mut oracle,
                step,
            )
        } else {
            let side = if step < 4 { 1 } else { next(&mut seed) % 2 };
            let before = oracle.carriers[&carrier].anchors[side];
            let after = if step == 2 {
                Anchor::Region(0)
            } else if step == 3 {
                Anchor::Exterior(0)
            } else {
                match before {
                    Anchor::Region(index) => Anchor::Exterior(1 - index),
                    Anchor::Exterior(index) => Anchor::Region(index),
                }
            };
            assert_ne!(before, after);
            let batch = lifecycle::replace_anchor(
                execution,
                &runtime,
                &current,
                carrier,
                side,
                before,
                after,
                &born,
                &format!("seeded-anchor-{step}"),
            );
            oracle.carriers.get_mut(&carrier).unwrap().anchors[side] = after;
            batch
        };
        let result = commit(execution, &runtime, batch);
        if step == 25 {
            let replacement = result.created_entity(&created(1, 8)).unwrap();
            assert_ne!(replacement, carriers[3]);
            carriers[3] = replacement;
            oracle.carriers.insert(
                replacement,
                Carrier {
                    vertices: BTreeSet::new(),
                    anchors: [Anchor::Region(1); 2],
                },
            );
        }
        oracle.check(execution, &runtime, &result);
        let actual = state(execution, &runtime, &result.snapshot, &oracle.groups());
        let split = actual.iter().flat_map(|group| &group.2).any(|event| {
            event.0 == result.commit.commit_id && event.1 == AggregateLineageKind::SplitFrom
        });
        let merge = actual.iter().flat_map(|group| &group.2).any(|event| {
            event.0 == result.commit.commit_id && event.1 == AggregateLineageKind::MergedInto
        });
        if split && !saw_split || merge && !saw_merge {
            audit(execution, &runtime, &result);
        }
        saw_split |= split;
        saw_merge |= merge;
        current = result.snapshot.clone();
        final_result = Some(result);
    }
    assert!(
        saw_split && saw_merge,
        "actual authored transitions must include both lineage kinds"
    );
    assert_eq!(
        state(execution, &runtime, &fork_snapshot, &original_groups),
        original
    );
    let final_result = final_result.unwrap();
    audit(execution, &runtime, &final_result);
    let expected = state(execution, &runtime, &current, &oracle.groups());
    let (_, recovered) = recover_with(execution, &runtime, || persisted(registry()));
    let recovered_head = snapshot_for_owner_branch(&recovered, &BranchId("main".into()));
    assert_eq!(
        state(execution, &recovered, &recovered_head, &oracle.groups()),
        expected
    );
    assert_eq!(
        demand_canonical_history_snapshot(execution, &runtime),
        demand_canonical_history_snapshot(execution, &recovered)
    );
}
