use super::super::damage;
use super::super::reuse_source_fixture::{
    frame, graphed, manifest, record, selected_source, walk_over, CLAIM, SOURCE_CHUNK,
    SOURCE_DECLARATION, SOURCE_LEAF, SOURCE_PUBLICATION,
};
use super::super::{ExtentRoute, Pending};
use super::*;
use crate::integrity_observation::OfflineIndeterminatePhysicalReason as Indeterminate;
use crate::integrity_observation::OfflinePhysicalDamageCause as Cause;
use crate::integrity_observation::{BoundedMediaWalk, OfflineIntegrityObservationLimits};

const UNROUTED: [u8; 24] = [40; 24];

fn aliased() -> Outcome {
    Outcome::Unknown(Unknown::PhysicalAliasNotReinspected)
}

fn bound() -> Outcome {
    Outcome::Indeterminate(Indeterminate::EntryBoundExceeded)
}

/// The rows of a walk over no rows that noted `outcome` for each frame.
fn rows_after(frames: &[ChildExpectation], outcome: &Outcome) -> Vec<Selected> {
    let mut walk = walk_over(Vec::new());
    for expected in frames {
        walk.note_outcome(expected, outcome);
    }
    graphed(walk).selected
}

/// What keeps the walk from one record that its route names hides that record
/// and no other: the record keeps a row, and the walk stays complete.
#[test]
fn a_record_whose_manifest_or_first_frame_was_not_read_keeps_a_row_with_that_outcome() {
    for unread in [frame(UNROUTED, 0), manifest(UNROUTED)] {
        for outcome in [aliased(), bound(), damage(Cause::ChecksumMismatch)] {
            let rows = rows_after(&[unread.clone()], &outcome);
            assert_eq!(rows.len(), 1, "{unread:?} {outcome:?}");
            assert_eq!(rows[0].record, UNROUTED);
            assert!(rows[0].fact.is_none());
            assert_eq!(rows[0].outcome, outcome);
            assert_eq!(rows[0].kind, None);
            assert_eq!((rows[0].path.as_str(), rows[0].generation), ("arena", 1));
            let coverage = coverage_after(&[(unread.clone(), outcome)]);
            assert_eq!(coverage, Coverage::Complete, "{unread:?}");
        }
        assert!(rows_after(&[unread], &Outcome::Intact).is_empty());
    }

    // A later frame says nothing of a record whose first frame was read and
    // was not a blob record's.
    assert!(rows_after(&[frame(UNROUTED, 144)], &aliased()).is_empty());

    // One record has one row, however often a frame of it is noted.
    let twice = [manifest(UNROUTED), frame(UNROUTED, 0), frame(UNROUTED, 0)];
    assert_eq!(rows_after(&twice, &aliased()).len(), 1);
    let mut read = walk_over(selected_source());
    read.note_outcome(&frame(record(2), 0), &damage(Cause::ScopeMismatch));
    let rows = graphed(read).selected;
    assert_eq!(rows.len(), selected_source().len());
    assert!(rows.iter().all(|row| row.outcome == Outcome::Intact));
}

#[test]
fn the_route_of_an_unread_record_says_whether_it_is_a_blob_record() {
    let routed = |class: RouteClass| {
        let mut walk = walk_over(Vec::new());
        walk.note_outcome(&frame(UNROUTED, 0), &aliased());
        walk.routes.classes.insert(UNROUTED, class);
        let rows = graphed(walk).selected;
        rows.into_iter().map(|row| row.kind).collect::<Vec<_>>()
    };
    for code in [1, 2, 4, 5, 10, 13] {
        let kind = FrameKind::declared(code);
        assert!(kind.is_some(), "kind {code}");
        assert_eq!(routed(RouteClass::Blob(code)), vec![kind], "kind {code}");
    }
    // A route that does not say a kind may route a blob record of any kind.
    assert_eq!(routed(RouteClass::Blob(200)), vec![None]);
    assert_eq!(routed(RouteClass::UnknownLegacy), vec![None]);
    for class in [
        RouteClass::Opaque,
        RouteClass::DerivedDirectory,
        RouteClass::BTreeNode,
    ] {
        assert!(routed(class).is_empty(), "{class:?}");
    }
}

fn scoped(family: Family, scope: ChildScope) -> ChildExpectation {
    ChildExpectation {
        family,
        scope,
        ..frame(UNROUTED, 0)
    }
}

fn tree(family: Family) -> ChildExpectation {
    let scope = ChildScope::Tree {
        tree: 1,
        block: 1,
        level: 0,
        capacity: 4,
        first: Vec::new(),
        last: Vec::new(),
    };
    scoped(family, scope)
}

/// The coverage of a walk that noted each outcome in turn.
fn coverage_after(noted: &[(ChildExpectation, Outcome)]) -> Coverage {
    let mut walk = walk_over(Vec::new());
    for (expected, outcome) in noted {
        walk.note_outcome(expected, outcome);
    }
    walk.coverage
}

/// Only what hides records that the walk cannot name cuts it short: a routing
/// block that was not read, and the entry bound.
#[test]
fn a_walk_is_cut_short_only_by_what_hides_records_it_cannot_name() {
    let above = tree(Family::RootRoutingBlock);
    for outcome in [aliased(), bound()] {
        assert_eq!(
            coverage_after(&[(above.clone(), outcome.clone())]),
            Coverage::CutShort(outcome)
        );
    }
    // Damage above a record is not damage of a claim on that record.
    assert_eq!(
        coverage_after(&[(above.clone(), damage(Cause::Framing))]),
        Coverage::CutShort(Outcome::Unknown(Unknown::ParentScopeUnavailable))
    );
    assert_eq!(
        coverage_after(&[(above.clone(), Outcome::Intact)]),
        Coverage::Complete
    );
    // The first cause stands.
    assert_eq!(
        coverage_after(&[(above.clone(), aliased()), (above, bound())]),
        Coverage::CutShort(aliased())
    );

    // No record lies below these, and an unread record keeps its own row.
    let page = ChildScope::Page {
        segment: 1,
        page: 1,
        pages: 1,
    };
    let free_space = ChildScope::FreeSpace {
        tree: 1,
        capacity: 4,
    };
    for beside in [
        tree(Family::SegmentMembershipBlock),
        scoped(Family::FreeSpaceMembershipBlock, free_space),
        scoped(Family::SegmentMembershipBlock, page),
        manifest(UNROUTED),
        frame(UNROUTED, 0),
        frame(UNROUTED, 144),
    ] {
        for outcome in [aliased(), bound(), damage(Cause::Framing)] {
            assert_eq!(
                coverage_after(&[(beside.clone(), outcome)]),
                Coverage::Complete,
                "{beside:?}"
            );
        }
    }

    let mut stopped = walk_over(Vec::new());
    stopped.note_walk_stopped(&bound());
    assert_eq!(stopped.coverage, Coverage::CutShort(bound()));
}

/// Arena bytes that no route accounts for hold no selected row: they keep a
/// row from being intact, and excuse no row that a complete walk calls absent.
#[test]
fn arena_bytes_that_no_route_accounts_for_hide_no_selected_row() {
    let finished = |arena_routes_intact: bool| {
        let mut rows = selected_source();
        rows.remove(SOURCE_PUBLICATION);
        let root = std::env::temp_dir();
        let limits = OfflineIntegrityObservationLimits::new(8, 4096, 6, 4, 0, 10_000, 4096);
        let started = std::time::Instant::now();
        let mut media = BoundedMediaWalk::new(limits.unwrap(), root.clone(), started);
        let rows = walk_over(rows).finish(&root, &mut media, arena_routes_intact);
        let outcome = |row: usize| rows[row].outcome().clone();
        (outcome(SOURCE_DECLARATION), outcome(CLAIM - 1))
    };
    let unaccounted = Outcome::Unknown(Unknown::ParentScopeUnavailable);
    let absent = damage(Cause::ScopeMismatch);
    assert_eq!(finished(true), (Outcome::Intact, absent.clone()));
    assert_eq!(finished(false), (unaccounted, absent));
}

/// An unread frame interrupts the record it is a frame of, and no other.
#[test]
fn an_unread_frame_interrupts_only_the_record_it_is_a_frame_of() {
    const ASSEMBLED: [u8; 24] = [41; 24];
    let assembling = || {
        let mut walk = walk_over(Vec::new());
        walk.pending = Some(Pending {
            record: ASSEMBLED,
            logical_bytes: 288,
            path: "arena".into(),
            generation: 1,
            bytes: Vec::new(),
            route: ExtentRoute {
                format: [0; 10],
                arena: 1,
                extent: 1,
                logical_bytes: 288,
                frames: Vec::new(),
            },
            interruption: None,
        });
        walk
    };
    let interruption = |walk: &BlobRecordWalk| {
        let pending = walk.pending.as_ref().expect("the record being assembled");
        pending.interruption.clone()
    };

    for other in [frame(UNROUTED, 0), frame(UNROUTED, 144), manifest(UNROUTED)] {
        let mut walk = assembling();
        walk.note_outcome(&other, &aliased());
        assert_eq!(interruption(&walk), None, "{other:?}");
    }
    let mut walk = assembling();
    walk.note_outcome(&frame(UNROUTED, 0), &aliased());
    let rows = graphed(walk).selected;
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].record, &rows[0].outcome), (UNROUTED, &aliased()));

    // The first interruption of the record stands, and gives it no second row.
    let mut walk = assembling();
    walk.note_outcome(&frame(ASSEMBLED, 144), &aliased());
    walk.note_outcome(&frame(ASSEMBLED, 144), &bound());
    assert_eq!(interruption(&walk), Some(aliased()));
    assert!(graphed(walk).selected.is_empty());
}

/// How the walk came to have no frame of a row.
#[derive(Debug, Clone, Copy)]
enum Unseen {
    /// A complete walk routed no frame of it.
    Absent,
    /// A complete walk routed no frame of it, and could not read the extent
    /// manifest of another record, which its route says is a chunk frame.
    AbsentBesideUnread,
    /// Its first frame was routed and could not be read.
    Unread,
    /// Its extent manifest was routed and could not be read.
    UnreadManifest,
    /// The walk stopped before it could visit the row.
    Unvisited,
}

/// The rows of `selected_source` by record, judged without the row at `lost`.
fn source_without(lost: usize, unseen: Unseen) -> impl Fn(usize) -> Outcome {
    let mut rows = selected_source();
    let lost = rows.remove(lost);
    let mut walk = walk_over(rows);
    match unseen {
        Unseen::Absent => {}
        Unseen::AbsentBesideUnread => {
            walk.note_outcome(&manifest(UNROUTED), &aliased());
            walk.routes.classes.insert(UNROUTED, RouteClass::Blob(2));
        }
        Unseen::Unread => walk.note_outcome(&frame(lost.record, 0), &aliased()),
        Unseen::UnreadManifest => walk.note_outcome(&manifest(lost.record), &aliased()),
        Unseen::Unvisited => walk.note_walk_stopped(&bound()),
    }
    let rows = graphed(walk).selected;
    move |position: usize| {
        let sought = record(position as u64 + 1);
        let row = rows.iter().find(|row| row.record == sought);
        row.expect("a row of the source").outcome.clone()
    }
}

/// A tree node and a publication name their rows by record and by session; the
/// gate ahead of those checks leaves them undecided only on a row that the
/// walk could not see.
#[test]
fn a_tree_is_undecided_on_a_row_the_walk_could_not_see_and_damaged_by_an_absent_one() {
    let seen = [
        (Unseen::Unread, aliased(), aliased()),
        (Unseen::UnreadManifest, aliased(), aliased()),
        (Unseen::Unvisited, bound(), bound()),
    ];
    // An unread record that is not the row hides nothing of it.
    for absent in [Unseen::Absent, Unseen::AbsentBesideUnread] {
        let outcome = source_without(SOURCE_CHUNK, absent);
        assert_eq!(outcome(SOURCE_LEAF), damage(Cause::Pointer), "{absent:?}");
        assert_eq!(outcome(SOURCE_PUBLICATION), Outcome::Intact, "{absent:?}");
        let outcome = source_without(SOURCE_DECLARATION, absent);
        let mismatch = damage(Cause::ScopeMismatch);
        assert_eq!(outcome(SOURCE_LEAF), mismatch, "{absent:?}");
        assert_eq!(outcome(SOURCE_PUBLICATION), mismatch, "{absent:?}");
        let outcome = source_without(SOURCE_LEAF, absent);
        assert_eq!(outcome(SOURCE_PUBLICATION), damage(Cause::Pointer));
    }

    for (unseen, leaf, publication) in seen {
        let outcome = source_without(SOURCE_CHUNK, unseen);
        assert_eq!(outcome(SOURCE_LEAF), leaf, "{unseen:?}");
        let outcome = source_without(SOURCE_DECLARATION, unseen);
        assert_eq!(outcome(SOURCE_LEAF), leaf, "{unseen:?}");
        assert_eq!(outcome(SOURCE_PUBLICATION), publication, "{unseen:?}");
        let outcome = source_without(SOURCE_LEAF, unseen);
        assert_eq!(outcome(SOURCE_PUBLICATION), publication, "{unseen:?}");
    }
}
