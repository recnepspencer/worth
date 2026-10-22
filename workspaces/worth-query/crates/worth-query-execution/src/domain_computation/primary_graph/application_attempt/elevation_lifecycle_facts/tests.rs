//! Exact lifecycle facts retain native optional-write guards without widening authority.
use super::*;
use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority};
use worth_relational::facade::identity::{PartitionId, RelationId};

type Fact = WorthQueryApplicationObservedFact;

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new(aspect).unwrap(),
        CanonicalFieldPath::single(FieldKey::new(field).unwrap()),
    )
}

struct Lifecycle {
    fields: [(AspectFieldLocator, AspectValue); 8],
    absent: AspectFieldLocator,
    target: EntityId,
    kind: KindId,
}

impl Lifecycle {
    fn new(review: bool) -> Self {
        Self {
            fields: [
                (locator("elevation", "identity"), AspectValue::UInt64(11)),
                (
                    locator("elevation", "reason"),
                    AspectValue::String("protect".into()),
                ),
                (locator("elevation", "status"), AspectValue::UInt64(2)),
                (locator("elevation", "not-before"), AspectValue::UInt64(3)),
                (locator("elevation", "not-after"), AspectValue::UInt64(9)),
                (locator("review", "identity"), AspectValue::UInt64(12)),
                (locator("review", "type"), AspectValue::UInt64(1)),
                (locator("review", "status"), AspectValue::UInt64(1)),
            ],
            absent: if review {
                locator("review", "reviewed-at")
            } else {
                locator("elevation", "closed-at")
            },
            target: entity(if review { 2 } else { 1 }),
            kind: KindId::new(if review { 2 } else { 1 }),
        }
    }

    fn facts(&self) -> Vec<Fact> {
        let mut facts = self
            .fields
            .iter()
            .enumerate()
            .map(|(index, (locator, value))| Fact::Field {
                entity_id: entity(if index < 5 { 1 } else { 2 }),
                kind: KindId::new(if index < 5 { 1 } else { 2 }),
                locator: locator.clone(),
                value: value.clone(),
            })
            .collect::<Vec<_>>();
        for (relation, anchor, incoming, endpoint) in [
            (1, 1, true, Some(4)),
            (2, 1, true, Some(5)),
            (3, 1, false, Some(6)),
            (4, 1, false, Some(3)),
            (5, 1, false, Some(2)),
            (6, 2, false, Some(3)),
            (7, 2, true, None),
        ] {
            facts.push(Fact::Adjacency {
                relation_kind: KindId::new(relation),
                anchor: entity(anchor),
                direction: if incoming {
                    WorthQueryApplicationAdjacencyDirection::Incoming
                } else {
                    WorthQueryApplicationAdjacencyDirection::Outgoing
                },
                maximum_work_units: 1,
                relations: endpoint
                    .into_iter()
                    .map(
                        |endpoint| super::super::fact::WorthQueryApplicationObservedRelation {
                            relation_id: RelationId::new(PartitionId::main(), relation as u64, 1),
                            from: entity(if incoming { endpoint } else { anchor }),
                            to: entity(if incoming { anchor } else { endpoint }),
                        },
                    )
                    .collect(),
            });
        }
        facts.push(Fact::AbsentField {
            entity_id: self.target,
            kind: self.kind,
            locator: self.absent.clone(),
        });
        facts.push(self.guard());
        facts
    }

    fn guard(&self) -> Fact {
        Fact::AspectPresence {
            entity_id: self.target,
            kind: self.kind,
            aspect: self.absent.aspect().aspect_key().clone(),
            present: true,
        }
    }

    fn accepts(&self, facts: &[Fact]) -> bool {
        let f = &self.fields;
        lifecycle_facts_are_exact(
            facts,
            WorthQueryElevationLifecycleFactExpectation {
                elevation: entity(1),
                review: entity(2),
                resource: entity(3),
                requester: entity(4),
                approver: WorthQueryExpectedLifecycleRelation::Present(entity(5)),
                grant: entity(6),
                reviewer: WorthQueryExpectedLifecycleRelation::Absent,
                elevation_identity: (&f[0].0, &f[0].1),
                reason: (&f[1].0, &f[1].1),
                status: (&f[2].0, &f[2].1),
                not_before: (&f[3].0, &f[3].1),
                not_after: (&f[4].0, &f[4].1),
                review_identity: (&f[5].0, &f[5].1),
                review_type: (&f[6].0, &f[6].1),
                review_status: (&f[7].0, &f[7].1),
                absent_fields: &[(self.target, &self.absent)],
                requester_relation: KindId::new(1),
                approver_relation: KindId::new(2),
                grant_relation: KindId::new(3),
                resource_relation: Some(KindId::new(4)),
                review_relation: KindId::new(5),
                review_scope_relation: KindId::new(6),
                reviewer_relation: KindId::new(7),
            },
        )
    }
}

#[test]
fn exact_close_and_review_facts_keep_the_optional_write_presence_guard() {
    for review in [false, true] {
        let lifecycle = Lifecycle::new(review);
        assert!(lifecycle.accepts(&lifecycle.facts()));
    }
}

#[test]
fn presence_guards_cannot_change_subject_kind_aspect_or_presence() {
    for review in [false, true] {
        let lifecycle = Lifecycle::new(review);
        let facts = lifecycle.facts();
        let aspect = lifecycle.absent.aspect().aspect_key().clone();
        for (entity_id, kind, aspect, present) in [
            (entity(99), lifecycle.kind, aspect.clone(), true),
            (lifecycle.target, KindId::new(99), aspect.clone(), true),
            (
                lifecycle.target,
                lifecycle.kind,
                AspectKey::new("other-lifecycle").unwrap(),
                true,
            ),
            (lifecycle.target, lifecycle.kind, aspect, false),
        ] {
            let mut wrong = facts.clone();
            *wrong.last_mut().unwrap() = Fact::AspectPresence {
                entity_id,
                kind,
                aspect,
                present,
            };
            assert!(!lifecycle.accepts(&wrong));
        }
    }
}

#[test]
fn duplicate_guards_and_unrelated_facts_do_not_widen_the_lifecycle_read_set() {
    let lifecycle = Lifecycle::new(false);
    for extra in [
        lifecycle.guard(),
        Fact::SourceEntity {
            entity_id: entity(99),
        },
    ] {
        let mut facts = lifecycle.facts();
        facts.push(extra);
        assert!(!lifecycle.accepts(&facts));
    }
}

#[test]
fn a_guard_cannot_substitute_another_lifecycle_field_contract() {
    let lifecycle = Lifecycle::new(false);
    let mut facts = lifecycle.facts();
    let last = facts.len() - 1;
    let other = locator("other-lifecycle", "closed-at");
    facts[last - 1] = Fact::AbsentField {
        entity_id: lifecycle.target,
        kind: lifecycle.kind,
        locator: other.clone(),
    };
    facts[last] = Fact::AspectPresence {
        entity_id: lifecycle.target,
        kind: lifecycle.kind,
        aspect: other.aspect().aspect_key().clone(),
        present: true,
    };
    assert!(!lifecycle.accepts(&facts));
}
