//! Perf Core Laws 1 and 12: resolving a held name reads only its own entry
//! and forward lineage path. Unrelated named records and unrelated commits
//! leave the exact work of a 3-hop resolution unchanged.
//!
//! The held face's history is real Phase 2 finalization. The unrelated named
//! records are canonical `Create`/`Retire` events in the same commits, since
//! finalizing tens of thousands of births per court would measure posture
//! validation rather than the index.

use crate::facade::history::CommitId;
use crate::facade::identity::LineageId;
use crate::facade::lineage::{
    PersistentNameBranchIndex, PersistentNameKey, PersistentNameResolutionMetrics,
    PersistentNameSelection,
};

use super::name_event_fixtures::{named_create, retire, synthetic_entity, synthetic_name};
use super::name_index_fixtures::{exact_key, main_branch, probes};
use super::name_world::{LineageWorld, Subject};

const UNRELATED_LINEAGE_BASE: u64 = 1 << 40;
const UNRELATED_EVENT_BASE: u64 = 1 << 41;

struct Scaled {
    head: u64,
    work: Vec<PersistentNameResolutionMetrics>,
}

/// Builds `records` unrelated named faces over `commits` unrelated commits
/// (retiring every fourth one a commit later), with the held face's three
/// split hops spread across that history.
fn scaled_world(records: u64, commits: u64) -> Scaled {
    let mut world = LineageWorld::new();
    let mut index = PersistentNameBranchIndex::new(main_branch(), None);
    index.apply(&world.log()[0]).unwrap();
    let held = world.create(Subject::Entity, "held");
    index.apply(&world.commit()).unwrap();
    let per_commit = records / commits;
    let (mut created, mut current, mut leaf) = (0, world.name_of(&held), held.clone());
    let mut hop_commits = Vec::new();
    for step in 0..commits {
        if [0, commits / 2, commits - 1].contains(&step) {
            let pieces = world.split(current, &["kept", "cut"]);
            index.apply(&world.commit()).unwrap();
            hop_commits.push(world.head().0);
            world.retire(world.name_of(&pieces[1]));
            (current, leaf) = (world.name_of(&pieces[0]), pieces[0].clone());
        }
        let batch = world.commit();
        let commit = batch.commit().0;
        let mut events = Vec::new();
        let mut holders = Vec::new();
        for ordinal in created..created + per_commit {
            let lineage = LineageId(UNRELATED_LINEAGE_BASE + ordinal);
            let event_id = UNRELATED_EVENT_BASE + 2 * ordinal;
            events.push(named_create(
                commit,
                event_id,
                lineage,
                synthetic_name(0xee, ordinal),
            ));
            holders.push((lineage, synthetic_entity(ordinal)));
            if ordinal >= per_commit && ordinal % 4 == 0 {
                let earlier = ordinal - per_commit;
                let lineage = LineageId(UNRELATED_LINEAGE_BASE + earlier);
                events.push(retire(
                    commit,
                    UNRELATED_EVENT_BASE + 2 * earlier + 1,
                    lineage,
                ));
            }
        }
        created += per_commit;
        index
            .apply(&batch.with_lineage_events(events, holders, []))
            .expect("unrelated history ingests");
    }
    assert_eq!(created, records);
    assert_eq!(index.name_count() as u64, records + 7);
    let (held, leaf): (PersistentNameKey, _) = (world.key_of(&held), world.key_of(&leaf));
    let head = world.head().0;
    hop_commits.push(head);
    let mut work = Vec::new();
    for basis in hop_commits {
        let outcome = index.resolve(held, CommitId(basis), &PersistentNameSelection::Exact);
        work.push(outcome.metrics);
        if basis == head {
            assert_eq!(exact_key(&outcome.result), leaf);
        }
    }
    Scaled { head, work }
}

#[test]
fn three_hop_resolution_work_is_flat_across_unrelated_records_and_commits() {
    let small = scaled_world(1_000, 50);
    let records = scaled_world(20_000, 50);
    let history = scaled_world(1_000, 500);
    let both = scaled_world(20_000, 500);
    // At head: held, 3 kept pieces, and 3 retired cut pieces are probed; every
    // name but the live leaf contributes its ending transition and edges.
    assert_eq!(small.work[3], probes(7, 6, 6));
    assert_eq!(small.work[0], probes(3, 1, 2), "first hop: two live pieces");
    for other in [&records, &history, &both] {
        assert_eq!(
            other.work, small.work,
            "work never depends on unrelated history"
        );
    }
    assert!(both.head > small.head + 400 && history.head > small.head + 400);
}
