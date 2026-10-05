//! Claims on rows that the walk could not read or did not visit. Unread is
//! not absent: a row with no frame contradicts a claim only in a walk that saw
//! every row that could be it.

use worth_foundational::PhysicalArtifactFamily as Family;

use super::super::super::record_walk::route_inventory::RouteClass;
use super::super::reuse_source_fixture::{
    chunk, claim, frame, graphed, manifest, record, released_source, row, selected_source,
    walk_over, witness, CHUNK, CLAIM, DESTINATION_DECLARATION, RELEASE_MANIFEST, SOURCE,
    SOURCE_CHUNK, SOURCE_DECLARATION, SOURCE_LEAF, SOURCE_PUBLICATION,
};
use super::super::{BlobRecordWalk, ChildExpectation, ChildScope};
use super::tests::mismatch;
use super::*;
use crate::integrity_observation::OfflineIndeterminatePhysicalReason as Indeterminate;

const LENGTH: u64 = CHUNK as u64;

/// A record that no row of the fixtures names.
const BESIDE: u64 = 90;

fn aliased() -> Outcome {
    Outcome::Unknown(Unknown::PhysicalAliasNotReinspected)
}

fn bound() -> Outcome {
    Outcome::Indeterminate(Indeterminate::EntryBoundExceeded)
}

/// How the walk came to have no frame of a row.
enum Unseen {
    /// The walk visited every routed record, and none is the row.
    Absent,
    /// The row's first frame was routed and could not be read.
    Unread(Outcome),
    /// The row's extent manifest was routed and could not be read.
    UnreadManifest(Outcome),
    /// The walk stopped before it visited every routed record.
    Unvisited(Outcome),
}

fn claim_of(walk: BlobRecordWalk) -> Outcome {
    let rows = graphed(walk).selected;
    let claim = rows.iter().find(|row| row.record == record(6));
    claim.expect("the claim row").outcome.clone()
}

/// The claim's outcome in a walk that has no frame of the row at `lost`.
fn claim_without(mut rows: Vec<Selected>, lost: usize, unseen: Unseen) -> Outcome {
    let lost = rows.remove(lost);
    let mut walk = walk_over(rows);
    match unseen {
        Unseen::Absent => {}
        Unseen::Unread(outcome) => walk.note_outcome(&frame(lost.record, 0), &outcome),
        Unseen::UnreadManifest(outcome) => walk.note_outcome(&manifest(lost.record), &outcome),
        Unseen::Unvisited(bound) => walk.note_walk_stopped(&bound),
    }
    claim_of(walk)
}

type Source = fn() -> Vec<Selected>;

/// The claim's outcome in a complete walk that has no frame of the row at
/// `lost` and could not read another record, routed as `class`.
fn claim_beside(source: Source, lost: usize, class: Option<RouteClass>) -> Outcome {
    claim_beside_unread(source, lost, class, &aliased())
}

/// As `claim_beside`, with `outcome` for what the walk could not read: the
/// other record's first frame or its extent manifest, which hide the same.
fn claim_beside_unread(
    source: Source,
    lost: usize,
    class: Option<RouteClass>,
    outcome: &Outcome,
) -> Outcome {
    let beside = |unread: ChildExpectation| {
        let mut rows = source();
        rows.remove(lost);
        let mut walk = walk_over(rows);
        walk.note_outcome(&unread, outcome);
        if let Some(class) = class {
            walk.routes.classes.insert(record(BESIDE), class);
        }
        claim_of(walk)
    };
    let claim = beside(frame(record(BESIDE), 0));
    assert_eq!(beside(manifest(record(BESIDE))), claim, "row {lost}");
    claim
}

fn witnessed_source() -> Vec<Selected> {
    let mut rows = selected_source();
    rows[CLAIM] = row(6, claim(Some(witness())));
    rows
}

#[test]
fn a_source_publication_that_was_not_read_is_not_a_released_one() {
    let sources: [Source; 2] = [selected_source, witnessed_source];
    for source in sources {
        for outcome in [aliased(), bound()] {
            let unread = Unseen::Unread(outcome.clone());
            assert_eq!(claim_without(source(), SOURCE_PUBLICATION, unread), outcome);
            let unread = Unseen::UnreadManifest(outcome.clone());
            assert_eq!(claim_without(source(), SOURCE_PUBLICATION, unread), outcome);
            let unvisited = Unseen::Unvisited(outcome.clone());
            assert_eq!(
                claim_without(source(), SOURCE_PUBLICATION, unvisited),
                outcome
            );
        }
        // A publication with no row in a walk that saw every routed record was
        // released, and no selected manifest says so.
        let absent = claim_without(source(), SOURCE_PUBLICATION, Unseen::Absent);
        assert_eq!(absent, mismatch());
        // A frame that was read and is damaged proves nothing for the claim.
        let damaged = Unseen::Unread(damage(Cause::ChecksumMismatch));
        assert_eq!(
            claim_without(source(), SOURCE_PUBLICATION, damaged),
            mismatch()
        );
    }
}

#[test]
fn a_row_under_a_selected_publication_that_was_not_read_is_not_an_absent_one() {
    for lost in [
        DESTINATION_DECLARATION,
        SOURCE_LEAF,
        SOURCE_CHUNK,
        SOURCE_DECLARATION,
    ] {
        let without = |unseen: Unseen| claim_without(selected_source(), lost, unseen);
        assert_eq!(without(Unseen::Unread(aliased())), aliased(), "row {lost}");
        let unread_manifest = without(Unseen::UnreadManifest(aliased()));
        assert_eq!(unread_manifest, aliased(), "row {lost}");
        assert_eq!(without(Unseen::Unvisited(bound())), bound(), "row {lost}");
        assert_eq!(without(Unseen::Absent), mismatch(), "row {lost}");
    }
}

#[test]
fn a_release_manifest_or_a_borrowed_chunk_that_was_not_read_is_not_an_absent_one() {
    for lost in [RELEASE_MANIFEST, SOURCE_CHUNK] {
        let without = |unseen: Unseen| claim_without(released_source(), lost, unseen);
        assert_eq!(without(Unseen::Unread(aliased())), aliased(), "row {lost}");
        let unread_manifest = without(Unseen::UnreadManifest(aliased()));
        assert_eq!(unread_manifest, aliased(), "row {lost}");
        assert_eq!(without(Unseen::Unvisited(bound())), bound(), "row {lost}");
        assert_eq!(without(Unseen::Absent), mismatch(), "row {lost}");
    }
}

/// A walk that was cut short leaves undecided only what it could not see: a
/// frame that was read still contradicts the claim.
#[test]
fn a_walk_cut_short_leaves_absence_undecided_and_hides_no_contradiction() {
    let routing_block = ChildExpectation {
        family: Family::RootRoutingBlock,
        scope: ChildScope::Tree {
            tree: 1,
            block: 1,
            level: 0,
            capacity: 4,
            first: Vec::new(),
            last: Vec::new(),
        },
        ..frame(record(BESIDE), 0)
    };
    let below_an_unread_block = |mut rows: Vec<Selected>, lost: usize| {
        rows.remove(lost);
        let mut walk = walk_over(rows);
        walk.note_outcome(&routing_block, &aliased());
        claim_of(walk)
    };
    for lost in [SOURCE_PUBLICATION, SOURCE_CHUNK, SOURCE_DECLARATION] {
        let outcome = below_an_unread_block(selected_source(), lost);
        assert_eq!(outcome, aliased(), "row {lost}");
    }
    let unread_manifest = below_an_unread_block(released_source(), RELEASE_MANIFEST);
    assert_eq!(unread_manifest, aliased());

    let another_digest = |mut rows: Vec<Selected>, lost: usize| {
        rows[SOURCE_CHUNK] = row(2, chunk(SOURCE, 0, LENGTH, [20; 32]));
        claim_without(rows, lost, Unseen::Unvisited(bound()))
    };
    for lost in [SOURCE_PUBLICATION, SOURCE_LEAF, DESTINATION_DECLARATION] {
        let outcome = another_digest(selected_source(), lost);
        assert_eq!(outcome, mismatch(), "row {lost}");
    }
    let released = another_digest(released_source(), RELEASE_MANIFEST);
    assert_eq!(released, mismatch());
}

/// The routes that say their record is no blob record.
const NO_BLOB_RECORD: [RouteClass; 3] = [
    RouteClass::Opaque,
    RouteClass::DerivedDirectory,
    RouteClass::BTreeNode,
];

/// A record that could not be read answers for itself alone, and may be a row
/// that a check finds by what its frame says only if its route says the kind
/// of that row, or says no kind.
#[test]
fn an_unread_record_is_only_a_row_that_its_record_and_its_kind_let_it_be() {
    let blob = |kind: u8| Some(RouteClass::Blob(kind));
    let no_blob_record = NO_BLOB_RECORD.map(Some);

    // Rows a claim names by record: no other record is one, whatever it is.
    let by_record: [(Source, usize); 3] = [
        (selected_source, SOURCE_CHUNK),
        (selected_source, SOURCE_LEAF),
        (released_source, SOURCE_CHUNK),
    ];
    let any = [None, Some(RouteClass::UnknownLegacy), blob(2), blob(3)];
    for (source, lost) in by_record {
        for class in any.into_iter().chain(no_blob_record) {
            let outcome = claim_beside(source, lost, class);
            assert_eq!(outcome, mismatch(), "row {lost} beside {class:?}");
        }
    }

    // Rows a claim finds by session. A frontier, an abandonment and a reuse
    // claim are reported under a declaration's family and are no declaration.
    for lost in [DESTINATION_DECLARATION, SOURCE_DECLARATION] {
        let beside = |class: Option<RouteClass>| claim_beside(selected_source, lost, class);
        assert_eq!(beside(None), aliased(), "row {lost}");
        assert_eq!(beside(Some(RouteClass::UnknownLegacy)), aliased());
        assert_eq!(beside(blob(1)), aliased(), "row {lost}");
        for other in [5, 6, 11, 15, 2, 13]
            .map(blob)
            .into_iter()
            .chain(no_blob_record)
        {
            assert_eq!(beside(other), mismatch(), "row {lost} beside {other:?}");
        }
    }

    // The manifest of a release. A failed ingest's manifest is none.
    let released: [(Source, usize); 2] = [
        (released_source, RELEASE_MANIFEST),
        (selected_source, SOURCE_PUBLICATION),
    ];
    for (source, lost) in released {
        let beside = |class: Option<RouteClass>| claim_beside(source, lost, class);
        assert_eq!(beside(None), aliased(), "row {lost}");
        assert_eq!(beside(blob(13)), aliased(), "row {lost}");
        for other in [7, 9, 2, 1].map(blob).into_iter().chain(no_blob_record) {
            assert_eq!(beside(other), mismatch(), "row {lost} beside {other:?}");
        }
    }
}

/// A record that was read and is damaged is no row unseen: it excuses no row
/// that a check finds by what its frame says.
#[test]
fn a_damaged_record_is_not_a_row_the_walk_could_not_see() {
    let damaged = damage(Cause::ChecksumMismatch);
    let lost: [(Source, usize); 4] = [
        (selected_source, DESTINATION_DECLARATION),
        (selected_source, SOURCE_DECLARATION),
        (selected_source, SOURCE_PUBLICATION),
        (released_source, RELEASE_MANIFEST),
    ];
    for (source, lost) in lost {
        for class in [None, Some(RouteClass::Blob(1)), Some(RouteClass::Blob(13))] {
            let outcome = claim_beside_unread(source, lost, class, &damaged);
            assert_eq!(outcome, mismatch(), "row {lost} beside {class:?}");
        }
    }
}
