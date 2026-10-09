use super::super::reuse_source_fixture::{
    chunk, declare_other, frame, graphed, record, row, selected_source, subsets, walk_over, walked,
    CHUNK, CLAIM, DESTINATION, DESTINATION_DECLARATION, DIGEST, OTHER, SCOPE, SOURCE, SOURCE_CHUNK,
    SOURCE_DECLARATION, SOURCE_LEAF, SOURCE_PUBLICATION, STORE,
};
use super::super::BlobRecordWalk;
use super::*;
use crate::integrity_observation::OfflineUnknownPhysicalReason as Unknown;

const LENGTH: u64 = CHUNK as u64;

const CONFLICTING_CHUNK: usize = CLAIM;
const QUARANTINE: usize = CONFLICTING_CHUNK + 1;

/// Every row of `collision` its quarantine is judged against.
const DEPENDENCIES: [usize; 6] = [
    SOURCE_DECLARATION,
    SOURCE_CHUNK,
    SOURCE_LEAF,
    SOURCE_PUBLICATION,
    DESTINATION_DECLARATION,
    CONFLICTING_CHUNK,
];

/// The dependencies a quarantine reaches by record. A row whose frame could
/// not be read keeps its record and its outcome but has no fact.
const BY_RECORD: [usize; 4] = [
    SOURCE_CHUNK,
    SOURCE_LEAF,
    SOURCE_PUBLICATION,
    CONFLICTING_CHUNK,
];

/// The destination wrote its own chunk under the source's digest key and the
/// quarantine names both occurrences.
fn collision() -> Vec<Selected> {
    let mut rows = selected_source();
    rows.truncate(CLAIM);
    rows.push(row(7, chunk(DESTINATION, 0, LENGTH, [21; 32])));
    rows.push(row(8, quarantine(0)));
    rows
}

fn quarantine(destination_ordinal: u64) -> BlobFact {
    BlobFact::DedupeQuarantine {
        store: STORE,
        scope: SCOPE,
        digest: DIGEST,
        chunk_size: CHUNK,
        source_publication: record(4),
        source_ordinal: 0,
        source_chunk: record(2),
        destination_session: DESTINATION,
        destination_ordinal,
        conflicting_chunk: record(7),
    }
}

type Reread = fn(&Selected) -> Result<Vec<u8>, Outcome>;

/// Chunk bytes that differ from row to row, as those of a collision do.
fn distinct(row: &Selected) -> Result<Vec<u8>, Outcome> {
    Ok(row.record.to_vec())
}

fn equal(_: &Selected) -> Result<Vec<u8>, Outcome> {
    Ok(vec![1])
}

fn unreadable(_: &Selected) -> Result<Vec<u8>, Outcome> {
    Err(damage(Cause::ChecksumMismatch))
}

/// The quarantine row's outcome once the claim graph has walked the rows and
/// the quarantine proof has reread the chunks it compares. The claim graph
/// settles every other row first and must leave a quarantine to weigh its own
/// undecided rows against what contradicts it.
fn quarantine_outcome(rows: Vec<Selected>, mut reread: Reread) -> Outcome {
    let mut rows = walked(rows);
    validate_reading(&mut rows, &Coverage::Complete, &mut reread);
    let quarantine = rows.iter().find(|row| row.record == record(8));
    quarantine.expect("the quarantine row").outcome.clone()
}

fn unavailable() -> Outcome {
    Outcome::Unknown(Unknown::WalCoverageUnavailable)
}

/// The rows' frames were read, but the walk could not settle the rows.
fn undecided(mut rows: Vec<Selected>, dependencies: &[usize]) -> Vec<Selected> {
    for dependency in dependencies {
        rows[*dependency].outcome = unavailable();
    }
    rows
}

/// The row's frame could not be read.
fn unread(mut rows: Vec<Selected>, dependency: usize) -> Vec<Selected> {
    rows[dependency].fact = None;
    rows[dependency].outcome = unavailable();
    rows
}

#[test]
fn two_proven_occurrences_are_a_collision_only_while_their_bytes_differ() {
    assert_eq!(quarantine_outcome(collision(), distinct), Outcome::Intact);
    assert_eq!(
        quarantine_outcome(collision(), equal),
        damage(Cause::ScopeMismatch)
    );
    assert_eq!(
        quarantine_outcome(collision(), unreadable),
        damage(Cause::ChecksumMismatch)
    );

    // The comparison reads the two chunks alone: an undecided row elsewhere
    // does not hide equal bytes.
    assert_eq!(
        quarantine_outcome(undecided(collision(), &[SOURCE_LEAF]), equal),
        damage(Cause::ScopeMismatch)
    );
}

#[test]
fn an_unavailable_row_a_quarantine_depends_on_is_not_pointer_damage() {
    for dependencies in subsets(&DEPENDENCIES).skip(1) {
        assert_eq!(
            quarantine_outcome(undecided(collision(), &dependencies), distinct),
            unavailable(),
            "rows {dependencies:?}"
        );
    }
    for dependency in DEPENDENCIES {
        let mut rows = collision();
        rows[dependency].outcome = damage(Cause::Truncation);
        assert_eq!(
            quarantine_outcome(rows, distinct),
            damage(Cause::Pointer),
            "a damaged row {dependency} proves nothing for the quarantine"
        );
    }
    for dependency in BY_RECORD {
        assert_eq!(
            quarantine_outcome(unread(collision(), dependency), distinct),
            unavailable(),
            "row {dependency}"
        );
    }
}

/// What frames that were read contradict, how it is written into `collision`,
/// and the rows whose unread frame hides it.
type Contradiction = (&'static str, fn(&mut Vec<Selected>), &'static [usize]);

const CONTRADICTIONS: [Contradiction; 9] = [
    (
        "a source chunk of another ordinal",
        |rows| rows[SOURCE_CHUNK] = row(2, chunk(SOURCE, 1, LENGTH, DIGEST)),
        &[SOURCE_CHUNK],
    ),
    (
        // The chunk is its own session's intact occurrence, and only the
        // publication names the source session.
        "a source chunk of another session",
        |rows| {
            rows[SOURCE_CHUNK] = row(2, chunk(OTHER, 0, LENGTH, DIGEST));
            declare_other(rows);
        },
        &[SOURCE_CHUNK, SOURCE_PUBLICATION],
    ),
    (
        "a source chunk of another digest",
        |rows| rows[SOURCE_CHUNK] = row(2, chunk(SOURCE, 0, LENGTH, [20; 32])),
        &[SOURCE_CHUNK],
    ),
    (
        // Every row of the source generation agrees on its store, so only
        // the quarantine says that the chunk is of another one.
        "a source generation of another store",
        |rows| {
            for source in &mut rows[..DESTINATION_DECLARATION] {
                if let Some(
                    BlobFact::Declaration { store, .. }
                    | BlobFact::Chunk { store, .. }
                    | BlobFact::Node { store, .. }
                    | BlobFact::Publication { store, .. },
                ) = source.fact.as_mut()
                {
                    *store = [2; 16];
                }
            }
        },
        &[SOURCE_CHUNK],
    ),
    (
        "a conflicting chunk of another session",
        |rows| {
            rows[CONFLICTING_CHUNK] = row(7, chunk(OTHER, 0, LENGTH, [21; 32]));
            declare_other(rows);
        },
        &[CONFLICTING_CHUNK],
    ),
    (
        "a conflicting chunk of another length than its declaration leaves",
        |rows| rows[CONFLICTING_CHUNK] = row(7, chunk(DESTINATION, 0, LENGTH - 1, [21; 32])),
        &[CONFLICTING_CHUNK],
    ),
    (
        "a quarantine past its destination's declared bytes",
        |rows| {
            rows[CONFLICTING_CHUNK] = row(7, chunk(DESTINATION, 1, LENGTH, [21; 32]));
            rows[QUARANTINE] = row(8, quarantine(1));
        },
        &[],
    ),
    (
        "a leaf edge that names another chunk",
        |rows| {
            if let Some(BlobFact::Node { entries, .. }) = rows[SOURCE_LEAF].fact.as_mut() {
                entries[0].record = record(90);
            }
        },
        &[SOURCE_LEAF, SOURCE_PUBLICATION],
    ),
    (
        "a conflicting chunk that is not selected",
        |rows| rows[CONFLICTING_CHUNK].record = record(90),
        &[],
    ),
];

/// A frame that was read contradicts the quarantine whatever the walk left
/// undecided, of that row or of any other; only the frame that could not be
/// read hides what it would have said. Rows that are not the two occurrences
/// have no bytes to compare, so the contradiction is pointer damage however a
/// reread would end. `an_unavailable_row_a_quarantine_depends_on_is_not_pointer_damage`
/// is the same world without the contradiction.
#[test]
fn a_contradiction_among_intact_rows_is_pointer_damage_whatever_other_row_is_unknown() {
    for (name, contradict, hidden_by) in CONTRADICTIONS {
        let contradicted = || {
            let mut rows = collision();
            contradict(&mut rows);
            rows
        };
        for reread in [distinct as Reread, equal, unreadable] {
            assert_eq!(
                quarantine_outcome(contradicted(), reread),
                damage(Cause::Pointer),
                "{name}"
            );
        }
        for dependencies in subsets(&DEPENDENCIES) {
            assert_eq!(
                quarantine_outcome(undecided(contradicted(), &dependencies), distinct),
                damage(Cause::Pointer),
                "{name}, rows {dependencies:?} undecided"
            );
        }
        for dependency in BY_RECORD {
            let expected = if hidden_by.contains(&dependency) {
                unavailable()
            } else {
                damage(Cause::Pointer)
            };
            assert_eq!(
                quarantine_outcome(unread(contradicted(), dependency), distinct),
                expected,
                "{name}, row {dependency} unread"
            );
        }
    }
}

/// A quarantine is undecided on a row that the walk could not read or did not
/// visit, and contradicted by one that a complete walk saw to be absent.
#[test]
fn a_row_the_walk_could_not_see_is_not_a_row_the_quarantine_lacks() {
    let aliased = Outcome::Unknown(Unknown::PhysicalAliasNotReinspected);
    let outcome = |walk: BlobRecordWalk| {
        let walk = graphed(walk);
        let (mut rows, coverage) = (walk.selected, walk.coverage);
        validate_reading(&mut rows, &coverage, &mut (distinct as Reread));
        let quarantine = rows.iter().find(|row| row.record == record(8));
        quarantine.expect("the quarantine row").outcome.clone()
    };
    for lost in [
        SOURCE_PUBLICATION,
        SOURCE_CHUNK,
        DESTINATION_DECLARATION,
        CONFLICTING_CHUNK,
    ] {
        let without = || {
            let mut rows = collision();
            let lost = rows.remove(lost);
            (walk_over(rows), lost.record)
        };
        let (absent, _) = without();
        assert_eq!(outcome(absent), damage(Cause::Pointer), "row {lost} absent");

        let (mut unread, record) = without();
        unread.note_outcome(&frame(record, 0), &aliased);
        assert_eq!(outcome(unread), aliased, "row {lost} unread");

        let (mut unvisited, _) = without();
        unvisited.note_walk_stopped(&aliased);
        assert_eq!(outcome(unvisited), aliased, "row {lost} unvisited");
    }
}
