//! Claims whose source publication a release removed.

use super::super::reuse_source_fixture::{
    chunk, claim, declare_other, declare_total, record, released_source, row, CHUNK, DIGEST, OTHER,
    RELEASED_CLAIM, RELEASE_DESCRIPTOR, RELEASE_MANIFEST, SOURCE, SOURCE_CHUNK, SOURCE_DECLARATION,
    SOURCE_LEAF,
};
use super::tests::{claim_outcome, mismatch};
use super::*;

const LENGTH: u64 = CHUNK as u64;

/// The release's first manifest and descriptor are selected.
fn removal_selected() -> Outcome {
    Outcome::Unknown(Unknown::WalCoverageUnavailable)
}

/// A manifest names the publication, and no more is selected of its removal.
fn release_selected() -> Outcome {
    Outcome::Unknown(Unknown::ParentScopeUnavailable)
}

fn unread() -> Outcome {
    Outcome::Unknown(Unknown::PhysicalAliasNotReinspected)
}

/// The claim's outcome once `change` is written into `released_source`.
fn release(change: impl FnOnce(&mut Vec<Selected>)) -> Outcome {
    let mut rows = released_source();
    change(&mut rows);
    claim_outcome(rows)
}

/// The release's first batch as one that dropped `records`.
fn dropping(rows: &mut [Selected], records: Vec<[u8; 24]>) {
    if let Some(BlobFact::ReleasedReclaimDescriptor { manifest_count, .. }) =
        rows[RELEASE_DESCRIPTOR].fact.as_mut()
    {
        *manifest_count = records.len().try_into().expect("a short drop set");
    }
    if let Some(BlobFact::ReleasedDropSetManifest { dropped, .. }) =
        rows[RELEASE_MANIFEST].fact.as_mut()
    {
        *dropped = records;
    }
}

#[test]
fn a_released_source_is_unknown_and_its_borrowed_chunk_is_the_witnessed_occurrence() {
    assert_eq!(
        claim_outcome(released_source()),
        removal_selected(),
        "selected V3/V2 provenance cannot invent selected WAL fate"
    );

    // The Store admits a released source on the claim's own witness, so the
    // chunk the claim borrows must still be that generation's occurrence.
    // Each chunk here is its own session's intact occurrence.
    let borrowing = |source_total: u64, borrowed: BlobFact| {
        let mut rows = released_source();
        declare_total(&mut rows, SOURCE_DECLARATION, source_total);
        rows[SOURCE_CHUNK] = row(2, borrowed);
        declare_other(&mut rows);
        claim_outcome(rows)
    };
    let another_session = chunk(OTHER, 0, LENGTH, DIGEST);
    assert_eq!(borrowing(LENGTH, another_session), mismatch());
    let another_digest = chunk(SOURCE, 0, LENGTH, [20; 32]);
    assert_eq!(borrowing(LENGTH, another_digest), mismatch());
    let another_ordinal = chunk(SOURCE, 1, LENGTH, DIGEST);
    assert_eq!(borrowing(2 * LENGTH, another_ordinal), mismatch());

    let mut damaged_chunk = released_source();
    damaged_chunk[SOURCE_CHUNK].outcome = damage(Cause::ChecksumMismatch);
    assert_eq!(claim_outcome(damaged_chunk), mismatch());

    // A selected claim protects the chunk it borrows from every release, and
    // the Store opens that chunk for every selected claim. This walk visited
    // every routed record, so a chunk with no row is absent from the store;
    // `unseen_tests` has the chunk that the walk could not see.
    let mut not_selected = released_source();
    not_selected.remove(SOURCE_CHUNK);
    assert_eq!(claim_outcome(not_selected), mismatch());

    let mut undecided_chunk = released_source();
    undecided_chunk[SOURCE_CHUNK].outcome = unread();
    assert_eq!(claim_outcome(undecided_chunk), unread());
    let mut unread_chunk = released_source();
    unread_chunk[SOURCE_CHUNK].fact = None;
    unread_chunk[SOURCE_CHUNK].outcome = unread();
    assert_eq!(claim_outcome(unread_chunk), unread());

    // The destination check reads no source row.
    let mut past_destination = released_source();
    if let Some(BlobFact::ReuseClaim { ordinal, .. }) =
        past_destination[RELEASED_CLAIM].fact.as_mut()
    {
        *ordinal = 1;
    }
    past_destination[SOURCE_CHUNK].fact = None;
    past_destination[SOURCE_CHUNK].outcome = unread();
    assert_eq!(claim_outcome(past_destination), mismatch());
}

/// Only a release removes a publication, and that release keeps a manifest
/// naming the publication selected while a claim protects one of its chunks.
/// Each walk here visited every routed record and read every frame but those
/// it says it could not, so a publication or a manifest with no row is absent
/// from the store; `unseen_tests` has the rows that a walk could not see.
#[test]
fn a_source_that_is_not_selected_was_released_by_a_selected_manifest_that_names_it() {
    let mut never_released = released_source();
    never_released.truncate(RELEASE_MANIFEST);
    assert_eq!(claim_outcome(never_released), mismatch());

    let mut unwitnessed = released_source();
    unwitnessed.truncate(RELEASE_MANIFEST);
    unwitnessed[RELEASED_CLAIM] = row(6, claim(None));
    assert_eq!(claim_outcome(unwitnessed), mismatch());

    // A row whose frame was read is not the manifest, whatever the walk left
    // undecided of it.
    let mut undecided_leaf = released_source();
    undecided_leaf.truncate(RELEASE_MANIFEST);
    undecided_leaf[SOURCE_LEAF].outcome = unread();
    assert_eq!(claim_outcome(undecided_leaf), mismatch());

    let manifest = |change: fn(&mut Selected)| release(|rows| change(&mut rows[RELEASE_MANIFEST]));
    for field in 0..MANIFEST_BASIS_FIELDS {
        assert_eq!(
            release(|rows| another_basis(&mut rows[RELEASE_MANIFEST], field)),
            mismatch(),
            "a manifest of another publication than the witnessed one, field {field}"
        );
    }
    let damaged = manifest(|row| row.outcome = damage(Cause::Truncation));
    assert_eq!(damaged, mismatch());

    // A row whose frame could not be read may be the manifest.
    let unread_manifest = manifest(|row| {
        row.fact = None;
        row.outcome = unread();
    });
    assert_eq!(unread_manifest, unread());
}

const MANIFEST_BASIS_FIELDS: usize = 8;

/// Rewrite one field of the publication that a release manifest names.
fn another_basis(manifest: &mut Selected, field: usize) {
    let Some(BlobFact::ReleasedDropSetManifest {
        store,
        object,
        session,
        generation,
        root,
        root_digest,
        publication_record,
        publication_digest,
        ..
    }) = manifest.fact.as_mut()
    else {
        return;
    };
    match field {
        0 => *publication_record = record(90),
        1 => *publication_digest = [24; 32],
        2 => *store = [2; 16],
        3 => *object = [24; 16],
        4 => *session = OTHER,
        5 => *generation = 2,
        6 => *root = record(90),
        _ => *root_digest = [24; 32],
    }
}

/// Selected records say that the release's first batch removed the publication
/// only while that batch's manifest and descriptor are both selected.
#[test]
fn only_the_first_selected_batch_of_a_release_says_what_removed_the_publication() {
    assert_eq!(release(|_| ()), removal_selected());
    assert_eq!(
        release(|rows| dropping(rows, vec![record(4), record(9)])),
        removal_selected()
    );

    // Checkpoints prune a release's first controls once a later batch is
    // selected; every batch's manifest still names the publication.
    assert_eq!(
        release(|rows| rows.truncate(RELEASE_DESCRIPTOR)),
        release_selected()
    );
    assert_eq!(
        release(|rows| dropping(rows, vec![record(9)])),
        release_selected(),
        "a later batch"
    );
    assert_eq!(
        release(|rows| dropping(rows, vec![record(2), record(4)])),
        release_selected(),
        "a batch that lists the borrowed chunk is not one the claim outlived"
    );
    let later_descriptor = release(|rows| {
        if let Some(BlobFact::ReleasedReclaimDescriptor { predecessor, .. }) =
            rows[RELEASE_DESCRIPTOR].fact.as_mut()
        {
            *predecessor = Some((record(9), [25; 32]));
        }
    });
    assert_eq!(later_descriptor, release_selected());
}

#[test]
fn an_unwitnessed_claim_on_a_released_source_still_names_the_chunk_it_borrows() {
    // Each chunk is its own session's intact occurrence, so only the claim
    // says that it is the wrong one.
    let unwitnessed = |source_total: u64, borrowed: BlobFact| {
        let mut rows = released_source();
        rows[RELEASED_CLAIM] = row(6, claim(None));
        declare_total(&mut rows, SOURCE_DECLARATION, source_total);
        rows[SOURCE_CHUNK] = row(2, borrowed);
        declare_other(&mut rows);
        claim_outcome(rows)
    };
    let named = chunk(SOURCE, 0, LENGTH, DIGEST);
    assert_eq!(unwitnessed(LENGTH, named), release_selected());
    let another_digest = chunk(SOURCE, 0, LENGTH, [20; 32]);
    assert_eq!(unwitnessed(LENGTH, another_digest), mismatch());
    let another_length = chunk(SOURCE, 0, LENGTH - 1, DIGEST);
    assert_eq!(unwitnessed(LENGTH - 1, another_length), mismatch());
    let another_ordinal = chunk(SOURCE, 1, LENGTH, DIGEST);
    assert_eq!(unwitnessed(2 * LENGTH, another_ordinal), mismatch());
    // Nothing selected names the session of an unwitnessed released source.
    let another_session = chunk(OTHER, 0, LENGTH, DIGEST);
    assert_eq!(unwitnessed(LENGTH, another_session), release_selected());
}
