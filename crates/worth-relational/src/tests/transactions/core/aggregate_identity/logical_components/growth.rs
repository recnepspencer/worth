//! Successful maintenance acquisition reports stay local beside unrelated carriers.
use super::{
    fixtures::*,
    oracle::{Anchor, Carrier, Oracle},
};
use std::collections::BTreeSet;

#[test]
fn one_case_delta_has_identical_actual_maintenance_acquisitions_beside_ten_and_ten_thousand_carriers(
) {
    let small = run(10);
    let large = run(10_000);
    assert!(
        small[0] > 0,
        "actual adjacency acquisition must be reported"
    );
    assert!(small[1] > 0, "actual entity acquisition must be reported");
    assert!(small[2] > 0, "actual relation acquisition must be reported");
    assert_eq!(
        large, small,
        "same completed law/cohort stages, not whole-census or charged work"
    );
}

fn run(unrelated: usize) -> [usize; 4] {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime();
    // Every unrelated carrier really exists and has both genuine isolated cases.
    // All setup and oracle reads precede the receipt whose counts are compared.
    let mut batch = WorkerIntentBatch::new("growth-corpus");
    for index in 0..unrelated + 2 {
        batch = batch.push(entity(1, index));
    }
    batch = batch
        .push(entity(6, 0))
        .push(entity(7, 0))
        .push(entity(8, 0));
    for index in 0..2 {
        batch = batch.push(edge(
            &format!("vertex-{index}"),
            INCIDENCE,
            EntityReference::Created(created(1, index)),
            EntityReference::Created(created(6, 0)),
        ));
        for side in [NEG_EXTERIOR, POS_EXTERIOR] {
            batch = batch.push(edge(
                &format!("exterior-{index}-{side}"),
                side,
                EntityReference::Created(created(1, index)),
                EntityReference::Created(created(8, 0)),
            ));
        }
    }
    let born = commit(execution, &runtime, batch);
    let fixed = [
        born.created_entity(&created(1, 0)).unwrap(),
        born.created_entity(&created(1, 1)).unwrap(),
    ];
    let probes = [
        born.created_entity(&created(1, 2)).unwrap(),
        born.created_entity(&created(1, unrelated + 1)).unwrap(),
    ];
    let mut oracle = Oracle::default();
    for carrier in fixed {
        oracle.carriers.insert(
            carrier,
            Carrier {
                vertices: BTreeSet::from([0]),
                anchors: [Anchor::Exterior(0); 2],
            },
        );
    }
    assert_eq!(oracle.groups().len(), 1);
    oracle.check(execution, &runtime, &born);
    let names = probes.map(|carrier| {
        [
            read(execution, &runtime, &born.snapshot, key(carrier, 0)).identity(),
            read(execution, &runtime, &born.snapshot, key(carrier, 1)).identity(),
        ]
    });
    for carrier in probes {
        for side in 0..2 {
            assert_eq!(
                read(execution, &runtime, &born.snapshot, key(carrier, side))
                    .members()
                    .cloned()
                    .collect::<Vec<_>>(),
                [key(carrier, side)]
            );
        }
    }
    let truth = runtime
        .read_truth()
        .read_snapshot(&born.snapshot, execution)
        .unwrap()
        .unwrap();
    let old = truth
        .relations()
        .iter()
        .find(|r| r.source == fixed[0] && r.kind.kind_id == KindId(POS_EXTERIOR))
        .unwrap()
        .relation_id;
    drop(truth);
    let changed = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("one-case-delta")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent { relation_id: old },
            )))
            .push(edge(
                "new-region",
                POS_REGION,
                EntityReference::Existing(fixed[0]),
                EntityReference::Existing(born.created_entity(&created(7, 0)).unwrap()),
            )),
    );
    oracle.carriers.get_mut(&fixed[0]).unwrap().anchors[1] = Anchor::Region(0);
    assert_eq!(
        oracle
            .groups()
            .iter()
            .map(BTreeSet::len)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([1, 3])
    );
    oracle.check(execution, &runtime, &changed);
    for (index, carrier) in probes.into_iter().enumerate() {
        for side in 0..2 {
            let actual = read(execution, &runtime, &changed.snapshot, key(carrier, side));
            assert_eq!(
                actual.members().cloned().collect::<Vec<_>>(),
                [key(carrier, side)]
            );
            assert_eq!(actual.identity(), names[index][side]);
        }
    }
    let actual = &changed.execution().complexity_delta;
    [
        actual.aggregate_maintenance_adjacency_lists,
        actual.aggregate_maintenance_entity_records,
        actual.aggregate_maintenance_relation_records,
        actual.aggregate_maintenance_aspect_states,
    ]
}
