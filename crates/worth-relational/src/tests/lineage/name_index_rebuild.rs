//! Arch 18: the derived name index is never authority. Destroying it and
//! rebuilding from the canonical committed log reproduces it exactly, and a
//! refused batch leaves it untouched.

use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use crate::aggregate_identity::AggregateGroupLineageEvent;
use crate::facade::history::CommitId;
use crate::facade::lineage::{
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameIndexDenial,
    PersistentNameKey, PersistentNameSelection,
};

use super::name_group_world::{member, GroupWorld};
use super::name_index_fixtures::{all, bounded, exact, main_branch, rebuild, side};
use super::name_world::{Keep, LineageWorld, Name, Subject};

/// A branch mixing entity, relation, and group naming in shared commits.
pub(super) fn mixed_history() -> (
    Vec<PersistentNameCommittedBatch>,
    PersistentNameBranchIndex,
    Vec<Name>,
) {
    let mut world = LineageWorld::new();
    let mut groups = GroupWorld::new();
    let mut incremental = PersistentNameBranchIndex::new(main_branch(), None);
    let mut log = Vec::new();
    let mut push = |batch: PersistentNameCommittedBatch, log: &mut Vec<_>| {
        incremental.apply(&batch).expect("committed batch ingests");
        log.push(batch);
    };
    push(world.log()[0].clone(), &mut log);
    let face = world.create(Subject::Entity, "face");
    let seam = world.create(Subject::Relation, "seam");
    let cutter = world.create(Subject::Entity, "cutter");
    let batch = world.commit();
    let born = groups.step(&[&[1, 2, 3, 4]]).unwrap();
    push(batch.with_group_transition(&born), &mut log);
    let cutter_name = world.name_of(&cutter);
    let face_pieces = world.split(world.name_of(&face), &["left", "right"]);
    world.bind_carriers(&face_pieces[0], &[cutter_name]);
    let seam_pieces = world.split(world.name_of(&seam), &["top", "bottom"]);
    let batch = world.commit();
    let split = groups.step(&[&[2], &[3, 4]]).unwrap();
    push(batch.with_group_transition(&split), &mut log);
    world.retire(world.name_of(&face_pieces[1]));
    world.merge(
        &[
            world.name_of(&seam_pieces[0]),
            world.name_of(&seam_pieces[1]),
        ],
        Keep::Recreated,
    );
    let batch = world.commit();
    let revived = groups.step(&[&[1], &[2, 3, 4]]).unwrap();
    push(batch.with_group_transition(&revived), &mut log);
    world.merge(
        &[world.name_of(&face_pieces[0]), cutter_name],
        Keep::MintNew,
    );
    let batch = world.commit();
    push(batch, &mut log);
    (log, incremental, vec![cutter_name, member(2), member(3)])
}

fn selections(carriers: &[Name]) -> Vec<PersistentNameSelection> {
    let mut selections = vec![exact(), all(), side("left"), side("top"), side("absent")];
    selections.extend(carriers.iter().map(|carrier| bounded(&[*carrier])));
    selections.push(bounded(carriers));
    selections
}

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
        log[1].lineage_events.clone(),
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
            detail: "only a live group can retire",
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
        batch.lineage_events.clone(),
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
