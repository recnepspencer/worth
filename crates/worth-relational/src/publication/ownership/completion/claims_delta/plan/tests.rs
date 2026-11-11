//! Compare the applied delta with independent whole-map replacement.
use super::*;
use crate::{
    history::data::{CanonicalRequiredOutputRelation, CommitId},
    identity::data::{KindId, PartitionId, VersionId},
};
use std::collections::BTreeMap;

fn relation(slot: u64, generation: u32, endpoint: u64) -> Claim {
    Claim::Relation(
        (&CanonicalRequiredOutputRelation {
            id: RelationId::new(PartitionId(7), slot, generation),
            kind: KindId(3),
            source: EntityId::new(PartitionId(7), endpoint, 1),
            target: EntityId::new(PartitionId(7), endpoint + 1, 1),
            birth: CommitId(generation as u64),
            before_write: None,
            after_write: VersionId(1),
        })
            .into(),
    )
}
fn stored(claim: &Claim, serial: u64) -> StoredClaim {
    let link = RelationId::new(PartitionId(9), serial, 1);
    match claim {
        Claim::Member(entity) => StoredClaim::Member {
            entity: *entity,
            link,
        },
        Claim::Relation(edge) => StoredClaim::Relation {
            edge: edge.clone(),
            witness: EntityId::new(PartitionId(9), serial, 1),
            link,
        },
    }
}
fn claim(value: &StoredClaim) -> Claim {
    match value {
        StoredClaim::Member { entity, .. } => Claim::Member(*entity),
        StoredClaim::Relation { edge, .. } => Claim::Relation(edge.clone()),
    }
}
#[test]
fn mixed_membership_relation_retirement_and_generation_changes_equal_replace_all() {
    for seed in 1..=64_u64 {
        let mut random = seed;
        let mut actual = BTreeMap::<RecordRef, StoredClaim>::new();
        let mut expected = BTreeMap::<RecordRef, Claim>::new();
        let mut serial = 0;
        for step in 0..100 {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            let next = if random & 1 == 0 {
                Claim::Member(EntityId::new(
                    PartitionId(7),
                    random % 24,
                    (random % 2 + 1) as u32,
                ))
            } else {
                relation(random % 24, (random % 3 + 1) as u32, (random >> 8) % 16)
            };
            if step % 19 == 0 {
                expected.clear();
            } else if random & 4 == 0 {
                expected.remove(&next.key());
            } else {
                expected.insert(next.key(), next);
            }
            let previous = actual.values().cloned().collect::<Vec<_>>();
            let desired = expected.values().cloned().collect::<Vec<_>>();
            let untouched = actual
                .iter()
                .filter(|(key, value)| expected.get(key).is_some_and(|next| claim(value) == *next))
                .map(|(key, value)| (key.clone(), value.link()))
                .collect::<BTreeMap<_, _>>();
            for change in changes(&previous, &desired) {
                match change {
                    Change::Retire(index) => {
                        actual.remove(&previous[index].key());
                    }
                    Change::Insert(index) => {
                        serial += 1;
                        actual.insert(desired[index].key(), stored(&desired[index], serial));
                    }
                    Change::Revise {
                        previous: old,
                        next,
                    } => {
                        let StoredClaim::Relation { witness, link, .. } = previous[old] else {
                            panic!("member identity changed meaning");
                        };
                        let Claim::Relation(edge) = &desired[next] else {
                            panic!("relation changed class");
                        };
                        actual.insert(
                            desired[next].key(),
                            StoredClaim::Relation {
                                edge: edge.clone(),
                                witness,
                                link,
                            },
                        );
                    }
                }
            }
            assert_eq!(
                actual
                    .iter()
                    .map(|(key, value)| (key.clone(), claim(value)))
                    .collect::<BTreeMap<_, _>>(),
                expected,
                "seed={seed}, step={step}: incremental graph differs from replace-all"
            );
            for (key, link) in untouched {
                assert_eq!(
                    actual[&key].link(),
                    link,
                    "unchanged physical claim was replaced"
                );
            }
        }
    }
}
#[test]
fn transient_write_proof_changes_do_not_rewrite_custody() {
    let edge = CanonicalRequiredOutputRelation {
        id: RelationId::new(PartitionId(7), 1, 1),
        kind: KindId(3),
        source: EntityId::new(PartitionId(7), 1, 1),
        target: EntityId::new(PartitionId(7), 2, 1),
        birth: CommitId(1),
        before_write: None,
        after_write: VersionId(1),
    };
    let custody = crate::publication::ownership::record::RelationClaim::from(&edge);
    let mut successor = edge.clone();
    successor.before_write = Some(VersionId(1));
    successor.after_write = VersionId(2);
    assert!(custody.matches(&successor));
    successor.target = EntityId::new(PartitionId(7), 3, 1);
    assert!(!custody.matches(&successor));
    let original = crate::publication::ownership::record::Edge {
        publication: CommitId(1),
        edge,
    };
    let bytes = serde_json::to_vec(&original).unwrap();
    let legacy: crate::publication::ownership::record::Edge =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        serde_json::to_vec(&legacy).unwrap(),
        bytes,
        "legacy witness receipt bytes survive unchanged"
    );
    assert_eq!(
        crate::publication::ownership::record::RelationClaim::from(&legacy.edge),
        custody
    );
}
