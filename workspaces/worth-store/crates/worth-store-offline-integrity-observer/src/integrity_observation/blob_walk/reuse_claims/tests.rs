use super::super::reuse_source_fixture::{
    chunk, claim, declare_other, declare_total, record, row, selected_source, subsets, walked,
    witness, CHUNK, CLAIM, DESTINATION_DECLARATION, DIGEST, OTHER, SOURCE, SOURCE_CHUNK,
    SOURCE_DECLARATION, SOURCE_LEAF, SOURCE_PUBLICATION,
};
use super::*;

const LENGTH: u64 = CHUNK as u64;

/// Every row of `selected_source` its claim is judged against.
const DEPENDENCIES: [usize; 5] = [
    SOURCE_DECLARATION,
    SOURCE_CHUNK,
    SOURCE_LEAF,
    SOURCE_PUBLICATION,
    DESTINATION_DECLARATION,
];

/// The dependencies a claim reaches by record. A row whose frame could not be
/// read keeps its record and its outcome but has no fact.
const BY_RECORD: [usize; 3] = [SOURCE_CHUNK, SOURCE_LEAF, SOURCE_PUBLICATION];

fn unavailable() -> Outcome {
    Outcome::Unknown(Unknown::WalCoverageUnavailable)
}

pub(super) fn mismatch() -> Outcome {
    damage(Cause::ScopeMismatch)
}

/// The claim row's outcome once the claim graph has walked the rows.
pub(super) fn claim_outcome(rows: Vec<Selected>) -> Outcome {
    let rows = walked(rows);
    let claim = rows.iter().find(|row| row.record == record(6));
    claim.expect("the claim row").outcome.clone()
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
fn selected_source_edge_and_destination_scope_authorize_one_reuse_claim() {
    assert_eq!(claim_outcome(selected_source()), Outcome::Intact);

    let mut rewritten = selected_source();
    rewritten[SOURCE_PUBLICATION].generation = 100;
    assert_eq!(
        claim_outcome(rewritten),
        Outcome::Intact,
        "rewritten source placement is not a later source publication"
    );

    let mut another_scope = selected_source();
    if let Some(BlobFact::ReuseClaim { scope, .. }) = another_scope[CLAIM].fact.as_mut() {
        *scope = [99; 32];
    }
    assert_eq!(claim_outcome(another_scope), mismatch());
}

#[test]
fn two_claims_for_one_ordinal_of_a_session_are_both_duplicates() {
    let mut rows = selected_source();
    rows.push(row(9, claim(None)));
    let rows = walked(rows);
    assert_eq!(rows[CLAIM].outcome, damage(Cause::DuplicateIdentity));
    assert_eq!(rows[CLAIM + 1].outcome, damage(Cause::DuplicateIdentity));

    // The identity is the session and the ordinal: a claim on another ordinal
    // is judged on its own, here as one past the declared total.
    let mut another_ordinal = claim(None);
    if let BlobFact::ReuseClaim { ordinal, .. } = &mut another_ordinal {
        *ordinal = 1;
    }
    let mut rows = selected_source();
    rows.push(row(9, another_ordinal));
    let rows = walked(rows);
    assert_eq!(rows[CLAIM].outcome, Outcome::Intact);
    assert_eq!(rows[CLAIM + 1].outcome, mismatch());
}

#[test]
fn an_unavailable_row_a_claim_depends_on_is_not_a_mismatch() {
    for dependencies in subsets(&DEPENDENCIES).skip(1) {
        assert_eq!(
            claim_outcome(undecided(selected_source(), &dependencies)),
            unavailable(),
            "rows {dependencies:?}"
        );
    }
    for dependency in DEPENDENCIES {
        let mut rows = selected_source();
        rows[dependency].outcome = damage(Cause::Truncation);
        assert_eq!(
            claim_outcome(rows),
            mismatch(),
            "a damaged row {dependency} proves nothing for the claim"
        );
    }
    for dependency in BY_RECORD {
        assert_eq!(
            claim_outcome(unread(selected_source(), dependency)),
            unavailable(),
            "row {dependency}"
        );
    }
}

/// What frames that were read contradict, how it is written into
/// `selected_source`, and the rows whose unread frame hides it.
type Contradiction = (&'static str, fn(&mut Vec<Selected>), &'static [usize]);

const CONTRADICTIONS: [Contradiction; 7] = [
    (
        "a claim past its destination's declared bytes",
        |rows| {
            if let Some(BlobFact::ReuseClaim { ordinal, .. }) = rows[CLAIM].fact.as_mut() {
                *ordinal = 1;
            }
        },
        &[],
    ),
    (
        "a borrowed chunk of another digest",
        |rows| rows[SOURCE_CHUNK] = row(2, chunk(SOURCE, 0, LENGTH, [20; 32])),
        &[SOURCE_CHUNK],
    ),
    (
        "a borrowed chunk of another length",
        |rows| rows[SOURCE_CHUNK] = row(2, chunk(SOURCE, 0, LENGTH - 1, DIGEST)),
        &[SOURCE_CHUNK],
    ),
    (
        "a borrowed chunk of another ordinal",
        |rows| rows[SOURCE_CHUNK] = row(2, chunk(SOURCE, 1, LENGTH, DIGEST)),
        &[SOURCE_CHUNK],
    ),
    (
        // The chunk is its own session's intact occurrence, and only the
        // publication names the source session of an unwitnessed claim.
        "a borrowed chunk of another session",
        |rows| {
            rows[SOURCE_CHUNK] = row(2, chunk(OTHER, 0, LENGTH, DIGEST));
            declare_other(rows);
        },
        &[SOURCE_CHUNK, SOURCE_PUBLICATION],
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
        "a source declared with another total",
        |rows| declare_total(rows, SOURCE_DECLARATION, 2 * LENGTH),
        &[SOURCE_PUBLICATION],
    ),
];

/// A frame that was read contradicts the claim whatever the walk left
/// undecided, of that row or of any other; only the frame that could not be
/// read hides what it would have said.
/// `an_unavailable_row_a_claim_depends_on_is_not_a_mismatch` is the same world
/// without the contradiction.
#[test]
fn a_contradiction_among_intact_rows_is_a_mismatch_whatever_other_row_is_unknown() {
    for (name, contradict, hidden_by) in CONTRADICTIONS {
        let contradicted = || {
            let mut rows = selected_source();
            contradict(&mut rows);
            rows
        };
        for dependencies in subsets(&DEPENDENCIES) {
            assert_eq!(
                claim_outcome(undecided(contradicted(), &dependencies)),
                mismatch(),
                "{name}, rows {dependencies:?} undecided"
            );
        }
        for dependency in BY_RECORD {
            let expected = if hidden_by.contains(&dependency) {
                unavailable()
            } else {
                mismatch()
            };
            assert_eq!(
                claim_outcome(unread(contradicted(), dependency)),
                expected,
                "{name}, row {dependency} unread"
            );
        }
    }
}

#[test]
fn a_witness_is_the_selected_publication_and_names_the_source_when_it_is_unknown() {
    let witnessed = |witness: ReuseSourceWitness| {
        let mut rows = selected_source();
        rows[CLAIM] = row(6, claim(Some(witness)));
        rows
    };
    assert_eq!(claim_outcome(witnessed(witness())), Outcome::Intact);

    let another_generation = ReuseSourceWitness {
        generation: 2,
        ..witness()
    };
    for dependencies in subsets(&DEPENDENCIES) {
        assert_eq!(
            claim_outcome(undecided(witnessed(another_generation), &dependencies)),
            mismatch(),
            "rows {dependencies:?} undecided"
        );
    }
    assert_eq!(
        claim_outcome(unread(witnessed(another_generation), SOURCE_PUBLICATION)),
        unavailable(),
        "only the publication's own frame contradicts a witness of it"
    );

    // The witness says whose chunk is borrowed when the publication cannot.
    let rows = unread(witnessed(witness()), SOURCE_PUBLICATION);
    assert_eq!(claim_outcome(rows), unavailable());
    let mut rows = unread(witnessed(witness()), SOURCE_PUBLICATION);
    rows[SOURCE_CHUNK] = row(2, chunk(OTHER, 0, LENGTH, DIGEST));
    declare_other(&mut rows);
    assert_eq!(claim_outcome(rows), mismatch());
}

/// The walk here visited every record the selected root routes, and a record
/// whose frame it could not read keeps a row, so a record with no row is
/// absent from the store. `unseen_tests` has the same rows unread and
/// unvisited, which are not absent.
#[test]
fn a_row_that_is_not_selected_under_a_selected_publication_is_a_mismatch() {
    for absent in [
        DESTINATION_DECLARATION,
        SOURCE_LEAF,
        SOURCE_CHUNK,
        SOURCE_DECLARATION,
    ] {
        let mut rows = selected_source();
        rows.remove(absent);
        assert_eq!(claim_outcome(rows), mismatch(), "row {absent}");
    }
}
