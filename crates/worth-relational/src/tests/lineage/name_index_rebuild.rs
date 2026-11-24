//! Arch 18: the derived name index is never authority. Destroying it and
//! rebuilding from the canonical committed log reproduces it exactly, and a
//! refused batch leaves it untouched.

use std::collections::BTreeSet;
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventRecordFieldsPort as _;
use worth_relational_lineage::internal_port::lineage::index::batch::PersistentNameCommittedBatchFixturePort as _;

use worth_foundational::facade::AggregateIdentity;

use crate::aggregate_identity::AggregateGroupLineageEvent;
use crate::facade::history::CommitId;
use crate::facade::lineage::{
    LineageEventKind, LineageSubject, PersistentNameBranchIndex, PersistentNameCommittedBatch,
    PersistentNameIndexDenial, PersistentNameKey,
};

use super::name_group_world::member;
use super::name_index_fixtures::{main_branch, mixed_history, rebuild, selections};
use super::name_world::{Keep, LineageWorld, Subject};

#[test]
fn destroyed_index_rebuilds_identical_state_and_answers_from_the_log_alone() {
    let (log, incremental, carriers) = mixed_history();
    let head = log.last().unwrap().commit().0;
    let rebuilt = rebuild(&log);
    assert_eq!(
        rebuilt, incremental,
        "rebuild reproduces the incremental index"
    );
    drop(incremental);
    let again = rebuild(&log);
    assert_eq!(again, rebuilt);
    let keys = rebuilt.name_keys();
    assert_eq!(
        keys.len(),
        11,
        "history names entities, relations and groups"
    );
    for subject in [
        |k: &PersistentNameKey| matches!(k, PersistentNameKey::Entity(_)),
        |k: &PersistentNameKey| matches!(k, PersistentNameKey::Relation(_)),
        |k: &PersistentNameKey| matches!(k, PersistentNameKey::Group(_)),
    ] {
        assert!(keys.iter().any(subject));
    }
    let mut compared = 0;
    for key in &keys {
        for selection in selections(&carriers) {
            for basis in 0..=head + 1 {
                let left = rebuilt.resolve(*key, CommitId(basis), &selection);
                let right = again.resolve(*key, CommitId(basis), &selection);
                assert_eq!(left, right, "{key:?} at {basis} under {selection:?}");
                compared += 1;
            }
        }
    }
    assert!(compared > 500);
}

fn group(byte: u8) -> AggregateIdentity {
    AggregateIdentity::from_bytes([byte; 32])
}

#[test]
fn refused_batches_leave_the_index_exactly_as_it_was() {
    let (log, mut index, _) = mixed_history();
    let before = index.clone();
    let head = index.head().unwrap();
    assert_eq!(
        index.apply(&log[1]),
        Err(PersistentNameIndexDenial::NonMonotonicCommit {
            head,
            next: log[1].commit(),
        })
    );
    let next = CommitId(head.0 + 1);
    let foreign = PersistentNameCommittedBatch::new(next).with_lineage_events(
        log[1].lineage_events_for_test().clone(),
        [],
        [],
    );
    assert!(matches!(
        index.apply(&foreign),
        Err(PersistentNameIndexDenial::ForeignEvent { expected_commit, .. }) if expected_commit == next
    ));
    // A valid birth followed by a refused retirement rolls back the birth.
    let half_valid = PersistentNameCommittedBatch::new(next).with_group_events([
        AggregateGroupLineageEvent::Born { group: group(0x71) },
        AggregateGroupLineageEvent::Retired { group: group(0x72) },
    ]);
    assert_eq!(
        index.apply(&half_valid),
        Err(PersistentNameIndexDenial::InvalidTransition {
            name: PersistentNameKey::Group(group(0x72)),
            detail: "only a group live before the commit can retire",
        })
    );
    let existing = before
        .name_keys()
        .into_iter()
        .find_map(|key| match key {
            PersistentNameKey::Group(group) => Some(group),
            _ => None,
        })
        .unwrap();
    let duplicate = PersistentNameCommittedBatch::new(next)
        .with_group_events([AggregateGroupLineageEvent::Born { group: existing }]);
    assert_eq!(
        index.apply(&duplicate),
        Err(PersistentNameIndexDenial::DuplicateBirth {
            name: PersistentNameKey::Group(existing),
        })
    );
    let dangling_edge = PersistentNameCommittedBatch::new(next).with_group_events([
        AggregateGroupLineageEvent::Born { group: group(0x73) },
        AggregateGroupLineageEvent::SplitFrom {
            fragment: group(0x73),
            source: group(0x74),
        },
    ]);
    assert!(index.apply(&dangling_edge).is_err());
    let unbound_carriers = PersistentNameCommittedBatch::new(next)
        .with_carriers(PersistentNameKey::Group(group(0x75)), [member(1)]);
    assert!(index.apply(&unbound_carriers).is_err());
    assert_eq!(index, before, "every refusal restored the prior index");

    let mut world = LineageWorld::new();
    world.create(Subject::Entity, "orphan");
    let batch = world.commit();
    let unbound = PersistentNameCommittedBatch::new(batch.commit()).with_lineage_events(
        batch.lineage_events_for_test().clone(),
        [],
        [],
    );
    let mut fresh = rebuild(&world.log()[..1]);
    let snapshot = fresh.clone();
    assert!(matches!(
        fresh.apply(&unbound),
        Err(PersistentNameIndexDenial::MissingHolder { .. })
    ));
    assert_eq!(fresh, snapshot);
    fresh.apply(&batch).expect("the bound batch ingests");
    assert_eq!(
        fresh.name_count(),
        BTreeSet::from_iter(fresh.name_keys()).len()
    );
}

/// Applies `batch`, which must refuse, and proves the index is untouched.
fn refuses(index: &mut PersistentNameBranchIndex, batch: &PersistentNameCommittedBatch) -> String {
    let before = index.clone();
    let denial = index.apply(batch).expect_err("corrupt batch refuses");
    assert_eq!(
        *index, before,
        "refusal of {denial:?} restored the prior index"
    );
    denial.to_string()
}

fn groups_at(commit: u64, events: Vec<AggregateGroupLineageEvent>) -> PersistentNameCommittedBatch {
    PersistentNameCommittedBatch::new(CommitId(commit)).with_group_events(events)
}

#[test]
fn corrupt_group_transitions_refuse_against_pre_commit_state() {
    use AggregateGroupLineageEvent::{Born, MergedInto, Retired, Revived, SplitFrom};
    let (a, b, c, d) = (group(0x81), group(0x82), group(0x83), group(0x84));
    let mut index = PersistentNameBranchIndex::new(main_branch(), None);
    index
        .apply(&groups_at(
            1,
            vec![Born { group: a }, Born { group: b }, Born { group: d }],
        ))
        .unwrap();
    index
        .apply(&groups_at(2, vec![Retired { group: d }]))
        .unwrap();
    let cases: Vec<(Vec<AggregateGroupLineageEvent>, &str)> = vec![
        (
            vec![MergedInto {
                absorbed: a,
                keeper: b,
            }],
            "retired by it",
        ),
        (
            vec![SplitFrom {
                fragment: b,
                source: d,
            }],
            "live before it",
        ),
        (
            vec![
                Born { group: c },
                SplitFrom {
                    fragment: c,
                    source: d,
                },
            ],
            "live before it",
        ),
        (
            vec![MergedInto {
                absorbed: d,
                keeper: b,
            }],
            "retired by it",
        ),
        (
            vec![
                Born { group: c },
                SplitFrom {
                    fragment: c,
                    source: c,
                },
            ],
            "live before it",
        ),
        (
            vec![SplitFrom {
                fragment: a,
                source: a,
            }],
            "loop to itself",
        ),
        (
            vec![
                Retired { group: a },
                MergedInto {
                    absorbed: a,
                    keeper: a,
                },
            ],
            "loop to itself",
        ),
        (
            vec![Born { group: c }, Retired { group: c }],
            "live before the commit",
        ),
        (
            vec![Revived { group: d }, Retired { group: d }],
            "live before the commit",
        ),
        (
            vec![SplitFrom {
                fragment: d,
                source: a,
            }],
            "reach a live group",
        ),
    ];
    for (events, reason) in cases {
        let detail = refuses(&mut index, &groups_at(3, events.clone()));
        assert!(detail.contains(reason), "{events:?} refused for `{detail}`");
    }
    // The canonical shapes still ingest: a kept split, then a merge.
    index
        .apply(&groups_at(
            3,
            vec![
                Born { group: c },
                SplitFrom {
                    fragment: c,
                    source: a,
                },
            ],
        ))
        .unwrap();
    index
        .apply(&groups_at(
            4,
            vec![
                Retired { group: a },
                MergedInto {
                    absorbed: a,
                    keeper: b,
                },
            ],
        ))
        .unwrap();
}

#[test]
fn corrupt_lineage_creates_and_subjects_refuse_atomically() {
    let mut world = LineageWorld::new();
    let a = world.create(Subject::Entity, "a");
    let b = world.create(Subject::Entity, "b");
    world.commit(); // 2
    let (a_name, b_name) = (world.name_of(&a), world.name_of(&b));
    world.merge(&[a_name, b_name], Keep::Recreated);
    let merge = world.commit(); // 3
    let mut index = rebuild(&world.log()[..2]);
    let kept_create = merge
        .lineage_events_for_test()
        .iter()
        .find(|event| event.kept_name().is_some())
        .expect("a recreated keeper emits a kept Create")
        .clone();
    let mut duplicated = merge.clone();
    duplicated
        .lineage_events_mut_for_test()
        .push(kept_create.clone());
    assert!(refuses(&mut index, &duplicated).contains("second name"));
    let mut dual = merge.clone();
    let birth_event = &world.log()[1].lineage_events_for_test()[0];
    let birth = birth_event.name_birth().cloned();
    for event in dual.lineage_events_mut_for_test() {
        if event.kept_name().is_some() {
            (*event.owner_name_birth_mut()) = birth.clone();
        }
    }
    assert!(refuses(&mut index, &dual).contains("both birth"));
    let mut foreign_subject = merge.clone();
    for event in foreign_subject.lineage_events_mut_for_test() {
        if event.kind() == LineageEventKind::Merge {
            (*event.owner_subject_mut()) = LineageSubject::Relation;
        }
    }
    assert!(refuses(&mut index, &foreign_subject).contains("event subject"));
    index.apply(&merge).expect("the canonical merge ingests");
}

#[test]
fn corrupt_restore_batches_refuse_atomically_leaving_index_unchanged() {
    use super::name_event_fixtures::synthetic_name;
    let mut world = LineageWorld::new();
    let a = world.create(Subject::Entity, "a");
    let b = world.create(Subject::Entity, "b");
    world.commit(); // 2
    let (a_name, b_name) = (world.name_of(&a), world.name_of(&b));
    world.retire(a_name);
    world.commit(); // 3
    world.restore(a_name, false);
    let restore_batch = world.commit(); // 4
    let mut index = rebuild(&world.log()[..3]);
    let birth_event = &world.log()[1].lineage_events_for_test()[0];
    let birth = birth_event.name_birth().cloned();
    let b_lin = world.log()[1]
        .lineage_events_for_test()
        .iter()
        .find(|e| e.name_birth().is_some_and(|n| n.identity() == b_name))
        .unwrap()
        .targets()[0];
    let mut check = |expected: &str, f: fn(&mut crate::facade::lineage::LineageEventRecord)| {
        let mut bad = restore_batch.clone();
        f(&mut bad.lineage_events_mut_for_test()[0]);
        let detail = refuses(&mut index, &bad);
        assert!(
            detail.contains(expected),
            "expected `{expected}`, got `{detail}`"
        );
    };
    check("kept_name", |e| (*e.owner_kept_name_mut()) = None);
    check("event subject", |e| {
        (*e.owner_subject_mut()) = LineageSubject::Relation
    });
    check("one source", |e| (*e.owner_sources_mut()).clear());
    check("one target", |e| (*e.owner_targets_mut()).clear());
    check("differ", |e| {
        (*e.owner_targets_mut()) = vec![(*e.owner_sources())[0]]
    });
    for (expected, mut bad) in [
        ("live name", restore_batch.clone()),
        ("does not exist", restore_batch.clone()),
        ("name_birth", restore_batch.clone()),
        ("second name", restore_batch.clone()),
        ("source lineage", restore_batch.clone()),
    ] {
        let ev = &mut bad.lineage_events_mut_for_test()[0];
        match expected {
            "live name" => {
                (*ev.owner_kept_name_mut()) = Some(b_name);
                (*ev.owner_sources_mut()) = vec![b_lin];
            }
            "does not exist" => (*ev.owner_kept_name_mut()) = Some(synthetic_name(0xee, 99)),
            "name_birth" => (*ev.owner_name_birth_mut()) = birth.clone(),
            "second name" => (*ev.owner_targets_mut()) = vec![b_lin],
            _ => (*ev.owner_sources_mut()) = vec![b_lin],
        }
        let detail = refuses(&mut index, &bad);
        assert!(
            detail.contains(expected),
            "expected `{expected}`, got `{detail}`"
        );
    }
    // Canonical restore ingests cleanly; a second retire + stale-source restore refuses.
    index.apply(&restore_batch).expect("canonical restore");
    world.retire(a_name);
    let retire2 = world.commit(); // 5
    index.apply(&retire2).expect("second retire");
    world.restore(a_name, false);
    let restore2 = world.commit(); // 6
    let mut stale = restore2.clone();
    (*stale.lineage_events_mut_for_test()[0].owner_sources_mut()) =
        (*restore_batch.lineage_events_for_test()[0].owner_sources()).clone();
    assert!(refuses(&mut index, &stale).contains("most recent retired"));
    index.apply(&restore2).expect("second canonical restore");
}
