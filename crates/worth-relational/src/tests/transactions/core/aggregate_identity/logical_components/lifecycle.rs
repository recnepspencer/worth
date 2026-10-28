//! Same-carrier cases, empty coupling births, grouping and full-key lifetime.
use super::{
    fixtures::*,
    oracle::{Anchor, Carrier, Oracle},
};
use std::collections::BTreeSet;
fn reference(kind: u32, index: usize) -> EntityReference {
    EntityReference::Created(created(kind, index))
}
pub(super) fn anchor_kind(side: usize, anchor: Anchor) -> u32 {
    match (side, anchor) {
        (0, Anchor::Region(_)) => NEG_REGION,
        (1, Anchor::Region(_)) => POS_REGION,
        (0, Anchor::Exterior(_)) => NEG_EXTERIOR,
        (1, Anchor::Exterior(_)) => POS_EXTERIOR,
        _ => unreachable!(),
    }
}
pub(super) fn anchor_reference(anchor: Anchor) -> EntityReference {
    match anchor {
        Anchor::Region(index) => reference(7, index),
        Anchor::Exterior(index) => reference(8, index),
    }
}
fn anchor_id(born: &CommitResult, anchor: Anchor) -> EntityId {
    match anchor {
        Anchor::Region(index) => born.created_entity(&created(7, index)).unwrap(),
        Anchor::Exterior(index) => born.created_entity(&created(8, index)).unwrap(),
    }
}
pub(super) fn replace_anchor(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    carrier: EntityId,
    side: usize,
    before: Anchor,
    after: Anchor,
    born: &CommitResult,
    label: &str,
) -> WorkerIntentBatch {
    let truth = runtime
        .read_truth()
        .read_snapshot(snapshot, execution)
        .unwrap()
        .unwrap();
    let old = truth
        .relations()
        .iter()
        .find(|edge| {
            edge.source == carrier && edge.kind.kind_id == KindId(anchor_kind(side, before))
        })
        .unwrap()
        .relation_id;
    WorkerIntentBatch::new(label)
        .push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent { relation_id: old },
        )))
        .push(edge(
            label,
            anchor_kind(side, after),
            EntityReference::Existing(carrier),
            EntityReference::Existing(anchor_id(born, after)),
        ))
}
#[test]
fn logical_cases_split_merge_and_transfer_without_invented_member_records_or_lineage() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    let mut batch = WorkerIntentBatch::new("logical-birth");
    for index in 0..4 {
        batch = batch.push(entity(1, index));
    }
    for index in 0..2 {
        batch = batch
            .push(entity(6, index))
            .push(entity(7, index))
            .push(entity(8, index));
    }
    for carrier in 0..4 {
        for side in 0..2 {
            batch = batch.push(edge(
                &format!("anchor-{carrier}-{side}"),
                anchor_kind(side, Anchor::Exterior(0)),
                reference(1, carrier),
                anchor_reference(Anchor::Exterior(0)),
            ));
        }
    }
    // 0 and1 touch only at vertex0. Carrier2 has a disjoint vertex;3 is
    // genuinely isolated even though both explicit anchors exist.
    for (carrier, vertex) in [(0, 0), (1, 0), (2, 1)] {
        batch = batch.push(edge(
            &format!("vertex-{carrier}"),
            INCIDENCE,
            reference(1, carrier),
            reference(6, vertex),
        ));
    }
    let born = commit(execution, &runtime, batch);
    let carriers = (0..4)
        .map(|i| born.created_entity(&created(1, i)).unwrap())
        .collect::<Vec<_>>();
    let mut oracle = Oracle::default();
    for (index, carrier) in carriers.iter().enumerate() {
        oracle.carriers.insert(
            *carrier,
            Carrier {
                vertices: match index {
                    0 | 1 => BTreeSet::from([0]),
                    2 => BTreeSet::from([1]),
                    _ => BTreeSet::new(),
                },
                anchors: [Anchor::Exterior(0); 2],
            },
        );
    }
    assert_eq!(
        oracle
            .groups()
            .iter()
            .map(BTreeSet::len)
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([1, 2, 4])
    );
    let truth = runtime
        .read_truth()
        .read_snapshot(&born.snapshot, execution)
        .unwrap()
        .unwrap();
    assert_eq!(
        truth
            .entities()
            .iter()
            .filter(|entity| entity.kind.kind_id == KindId(1))
            .count(),
        4,
        "logical cases do not allocate writable carrier records"
    );
    drop(truth);
    oracle.check(execution, &runtime, &born);
    let original = read(execution, &runtime, &born.snapshot, key(carriers[0], 0)).identity();
    assert_eq!(
        read(execution, &runtime, &born.snapshot, key(carriers[0], 1)).identity(),
        original,
        "full-key self exclusion must retain the other side of the same carrier"
    );
    let split = commit(
        execution,
        &runtime,
        replace_anchor(
            execution,
            &runtime,
            &born.snapshot,
            carriers[0],
            1,
            Anchor::Exterior(0),
            Anchor::Region(0),
            &born,
            "case-only-split",
        ),
    );
    oracle.carriers.get_mut(&carriers[0]).unwrap().anchors[1] = Anchor::Region(0);
    oracle.check(execution, &runtime, &split);
    assert_ne!(
        read(execution, &runtime, &split.snapshot, key(carriers[0], 0)).identity(),
        read(execution, &runtime, &split.snapshot, key(carriers[0], 1)).identity()
    );
    let truth = runtime
        .read_truth()
        .read_snapshot(&split.snapshot, execution)
        .unwrap()
        .unwrap();
    assert_eq!(
        truth
            .relations()
            .iter()
            .filter(|edge| edge.source == carriers[0]
                && edge.kind.kind_id == descriptor().membership_kind)
            .count(),
        2,
        "distinct case cohorts require two grouped memberships for one actual carrier"
    );
    drop(truth);
    let merged = commit(
        execution,
        &runtime,
        replace_anchor(
            execution,
            &runtime,
            &split.snapshot,
            carriers[0],
            1,
            Anchor::Region(0),
            Anchor::Exterior(0),
            &born,
            "case-only-merge",
        ),
    );
    oracle.carriers.get_mut(&carriers[0]).unwrap().anchors[1] = Anchor::Exterior(0);
    oracle.check(execution, &runtime, &merged);
    let mut transfer = WorkerIntentBatch::new("whole-cohort-region-transfer");
    for carrier in carriers.iter().take(2) {
        for side in 0..2 {
            transfer.intents.extend(
                replace_anchor(
                    execution,
                    &runtime,
                    &merged.snapshot,
                    *carrier,
                    side,
                    Anchor::Exterior(0),
                    Anchor::Region(1),
                    &born,
                    &format!("transfer-{}-{side}", carrier.local_slot_value()),
                )
                .intents,
            );
        }
    }
    let transferred = commit(execution, &runtime, transfer);
    for carrier in carriers.iter().take(2) {
        oracle.carriers.get_mut(carrier).unwrap().anchors = [Anchor::Region(1); 2];
    }
    oracle.check(execution, &runtime, &transferred);
    assert_eq!(
        read(
            execution,
            &runtime,
            &transferred.snapshot,
            key(carriers[0], 0)
        )
        .identity(),
        original
    );
    assert!(
        read(
            execution,
            &runtime,
            &transferred.snapshot,
            key(carriers[0], 0)
        )
        .lineage()
        .all(|event| event.occurrence() != transferred.commit.commit_id),
        "only qualification changed; full member keys and cohort stayed equal"
    );
    let least = *carriers.iter().take(2).min().unwrap();
    let truth = runtime
        .read_truth()
        .read_snapshot(&transferred.snapshot, execution)
        .unwrap()
        .unwrap();
    let mut removal = WorkerIntentBatch::new("retire-least-logical-carrier");
    for relation in truth.relations().iter().filter(|relation| {
        relation.source == least
            && [
                INCIDENCE,
                NEG_REGION,
                POS_REGION,
                NEG_EXTERIOR,
                POS_EXTERIOR,
            ]
            .contains(&relation.kind.kind_id.as_u32())
    }) {
        removal = removal.push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: relation.relation_id,
            },
        )));
    }
    drop(truth);
    removal = removal.push(MutationIntent::Entity(EntityMutationIntent::Delete(
        DeleteEntityIntent { entity_id: least },
    )));
    let removed = commit(execution, &runtime, removal);
    oracle.carriers.remove(&least);
    oracle.check(execution, &runtime, &removed);
    let remaining = *carriers
        .iter()
        .take(2)
        .find(|carrier| **carrier != least)
        .unwrap();
    assert_eq!(
        read(execution, &runtime, &removed.snapshot, key(remaining, 0)).identity(),
        original,
        "least-carrier removal retains the genuine surviving heir, not least-member naming"
    );
}
