//! The head a pending release replays must be the one its predecessor link
//! names. Above a NoRelease checkpoint this pairing is the only join between
//! a successor descriptor and the head tree it was built on.

use worth_store_physical_format::{
    PersistedRecordIdentity, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
    ReleasedDropPredecessorV1,
};

use super::replaced_head_matches;

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([8; 16], ordinal).unwrap()
}

fn head(
    object: u8,
    descriptor: u64,
    predecessor: Option<ReleasedDropPredecessorV1>,
    cumulative: u64,
    terminal: bool,
) -> ReleaseCustodyHeadEntryV1 {
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([object; 16], 1).unwrap(),
        record(descriptor),
        [descriptor as u8; 32],
        record(descriptor + 1),
        [2; 32],
        record(descriptor + 2),
        [3; 32],
        [4; 32],
        predecessor,
        4,
        cumulative,
        terminal,
    )
    .unwrap()
}

fn link(prior: ReleaseCustodyHeadEntryV1) -> ReleasedDropPredecessorV1 {
    ReleasedDropPredecessorV1::new(prior.descriptor_record(), prior.descriptor_frame_sha256())
        .unwrap()
}

#[test]
fn successor_replaces_exactly_the_live_head_its_predecessor_wrote() {
    let first = head(1, 10, None, 3, false);
    let predecessor = Some(link(first));
    let next = head(1, 20, predecessor, 5, false);
    assert!(replaced_head_matches(
        predecessor,
        Some(first),
        next,
        false,
        2
    ));
    // The replay found no head for the object: the predecessor the
    // descriptor names was never in the tree this release was built on.
    assert!(!replaced_head_matches(predecessor, None, next, false, 2));
    assert!(!replaced_head_matches(predecessor, None, next, true, 5));
    // The replayed head is another descriptor, another object, a terminal
    // head, or a count this manifest does not advance to the next entry.
    for fabricated in [
        head(1, 30, None, 3, false),
        head(2, 10, None, 3, false),
        head(1, 10, None, 3, true),
        head(1, 10, None, 4, false),
    ] {
        assert!(!replaced_head_matches(
            predecessor,
            Some(fabricated),
            next,
            false,
            2
        ));
    }
}

#[test]
fn first_release_replaces_no_head_and_drops_its_source_publication() {
    let next = head(1, 20, None, 2, false);
    assert!(replaced_head_matches(None, None, next, true, 2));
    assert!(!replaced_head_matches(None, None, next, false, 2));
    assert!(!replaced_head_matches(None, None, next, true, 3));
    // A first release that replayed a head would overwrite custody it does
    // not extend.
    let existing = head(1, 10, None, 3, false);
    assert!(!replaced_head_matches(None, Some(existing), next, true, 2));
}
