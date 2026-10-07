use super::super::super::blob_record::FrameKind;
use super::super::coverage::Coverage;
use super::super::damage;
use super::super::reuse_source_fixture::unread as unread_kind;
use super::*;
use crate::integrity_observation::OfflineIndeterminatePhysicalReason as Indeterminate;
use crate::integrity_observation::OfflineUnknownPhysicalReason as Unknown;

const SESSION: [u8; 16] = [2; 16];
const CHUNK: u64 = 64 << 10;
const DIGEST: [u8; 32] = [6; 32];

fn leaf(index: u64) -> EdgePosition<'static> {
    EdgePosition {
        session: &SESSION,
        kind: 1,
        level: 0,
        index: Some(index),
    }
}

fn edge() -> BlobEdge {
    BlobEdge {
        digest: DIGEST,
        record: [5; 24],
        covered: CHUNK,
    }
}

fn chunk(session: [u8; 16], ordinal: u64) -> BlobFact {
    BlobFact::Chunk {
        store: [1; 16],
        session,
        ordinal,
        chunk_size: CHUNK as u32,
        length: CHUNK,
        digest: DIGEST,
    }
}

fn reuse_claim(session: [u8; 16], ordinal: u64, length: u64, digest: [u8; 32]) -> BlobFact {
    BlobFact::ReuseClaim {
        store: [1; 16],
        session,
        ordinal,
        scope: [4; 32],
        chunk_size: CHUNK as u32,
        length,
        digest,
        chunk_record: [9; 24],
        source_publication: [10; 24],
        source_ordinal: 0,
        source_witness: None,
    }
}

fn selected(record: [u8; 24], fact: BlobFact, outcome: Outcome) -> Selected {
    Selected {
        record,
        path: "arena".into(),
        generation: 1,
        kind: Some(FrameKind::of(&fact)),
        fact: Some(fact),
        outcome,
        route: None,
    }
}

fn node(session: [u8; 16], level: u8, index: u64) -> BlobFact {
    BlobFact::Node {
        store: [1; 16],
        session,
        kind: 1,
        level,
        index,
        covered: CHUNK,
        digest: DIGEST,
        frame_digest: [7; 32],
        entries: Vec::new(),
    }
}

#[test]
fn a_leaf_edge_names_its_sessions_own_chunk_or_reuse_claim_for_that_ordinal() {
    assert!(edge_names_child(&leaf(3), &edge(), &chunk(SESSION, 3)));
    assert!(edge_names_child(
        &leaf(3),
        &edge(),
        &reuse_claim(SESSION, 3, CHUNK, DIGEST)
    ));

    // The chunk a reuse claim borrows belongs to its source session; a tree
    // never names it directly.
    assert!(!edge_names_child(&leaf(3), &edge(), &chunk([8; 16], 3)));
    assert!(!edge_names_child(&leaf(3), &edge(), &chunk(SESSION, 2)));
    for wrong in [
        reuse_claim([8; 16], 3, CHUNK, DIGEST),
        reuse_claim(SESSION, 2, CHUNK, DIGEST),
        reuse_claim(SESSION, 3, CHUNK - 1, DIGEST),
        reuse_claim(SESSION, 3, CHUNK, [0; 32]),
    ] {
        assert!(!edge_names_child(&leaf(3), &edge(), &wrong), "{wrong:?}");
    }
    assert!(!edge_names_child(&leaf(3), &edge(), &node(SESSION, 0, 3)));
}

#[test]
fn an_unavailable_reuse_claim_under_a_leaf_is_uncertainty_not_pointer_damage() {
    let unavailable = Outcome::Unknown(Unknown::WalCoverageUnavailable);
    let claim = selected(
        edge().record,
        reuse_claim(SESSION, 3, CHUNK, DIGEST),
        unavailable.clone(),
    );
    let mut leaf = node(SESSION, 0, 0);
    if let BlobFact::Node { entries, .. } = &mut leaf {
        entries.push(edge());
    }
    let rows = [claim];
    let found = RowIndex::new(&rows, &Coverage::Complete);
    assert_eq!(
        uncertain_dependency(&leaf, &rows, &found),
        Some(unavailable)
    );
}

/// A fact-less row whose frame could not be read and whose kind the walk
/// could not tell.
fn unread(record: [u8; 24], outcome: Outcome) -> Selected {
    unread_kind(record, None, outcome)
}

fn declaration() -> BlobFact {
    BlobFact::Declaration {
        store: [1; 16],
        session: SESSION,
        frame_digest: [3; 32],
        object: [4; 16],
        scope: [4; 32],
        chunk_size: CHUNK as u32,
        total: CHUNK,
        max_checkpoint_sequence: 9,
    }
}

/// The gate ahead of the checks that read rows by record and by session: a
/// row that is absent is undecided only where the walk could not see it.
#[test]
fn a_row_the_walk_could_not_see_is_an_uncertain_dependency_and_an_absent_one_is_not() {
    let aliased = Outcome::Unknown(Unknown::PhysicalAliasNotReinspected);
    let bound = Outcome::Indeterminate(Indeterminate::EntryBoundExceeded);
    let cut_short = Coverage::CutShort(bound.clone());
    let mut leaf = node(SESSION, 0, 0);
    if let BlobFact::Node { entries, .. } = &mut leaf {
        entries.push(edge());
    }
    let gate = |fact: &BlobFact, rows: &[Selected], coverage: &Coverage| {
        uncertain_dependency(fact, rows, &RowIndex::new(rows, coverage))
    };
    let declared = || selected([6; 24], declaration(), Outcome::Intact);
    let child = || selected(edge().record, chunk(SESSION, 0), Outcome::Intact);

    // The leaf's chunk, which it names by record.
    let no_chunk = [declared(), unread([8; 24], aliased.clone())];
    assert_eq!(gate(&leaf, &no_chunk, &Coverage::Complete), None);
    assert_eq!(gate(&leaf, &no_chunk, &cut_short), Some(bound.clone()));
    let unread_chunk = [declared(), unread(edge().record, aliased.clone())];
    assert_eq!(
        gate(&leaf, &unread_chunk, &Coverage::Complete),
        Some(aliased.clone())
    );

    // The leaf's declaration, which it names by session: a frame that could
    // not be read may be it, unless its kind says that it is not.
    let no_declaration = [child()];
    assert_eq!(gate(&leaf, &no_declaration, &Coverage::Complete), None);
    assert_eq!(
        gate(&leaf, &no_declaration, &cut_short),
        Some(bound.clone())
    );
    let unread_frame = [child(), unread([8; 24], aliased.clone())];
    assert_eq!(
        gate(&leaf, &unread_frame, &Coverage::Complete),
        Some(aliased.clone())
    );
    let of_kind = |kind: u8| {
        let unread = unread_kind([8; 24], Some(kind), aliased.clone());
        gate(&leaf, &[child(), unread], &Coverage::Complete)
    };
    assert_eq!(of_kind(1), Some(aliased.clone()));
    // A frontier, an abandonment and a reuse claim are reported under a
    // declaration's family and are no declaration.
    for other in [5, 6, 11, 15, 2, 13] {
        assert_eq!(of_kind(other), None, "kind {other}");
    }

    // A frame that names its declaration by record is not left undecided by
    // an unread frame of another record.
    let abandoned = BlobFact::Abandoned {
        store: [1; 16],
        frame_digest: [9; 32],
        session: SESSION,
        declaration_record: edge().record,
        declaration_digest: [3; 32],
        expiry_checkpoint: None,
    };
    assert_eq!(gate(&abandoned, &unread_frame, &Coverage::Complete), None);
    let no_rows = [unread([8; 24], aliased)];
    assert_eq!(gate(&abandoned, &no_rows, &Coverage::Complete), None);
    assert_eq!(gate(&abandoned, &no_rows, &cut_short), Some(bound));
}

#[test]
fn a_branch_edge_names_the_next_lower_node_of_its_session() {
    let branch = EdgePosition {
        session: &SESSION,
        kind: 2,
        level: 1,
        index: Some(3),
    };
    assert!(edge_names_child(&branch, &edge(), &node(SESSION, 0, 3)));
    assert!(!edge_names_child(&branch, &edge(), &node(SESSION, 1, 3)));
    assert!(!edge_names_child(&branch, &edge(), &node(SESSION, 0, 2)));
    assert!(!edge_names_child(&branch, &edge(), &node([8; 16], 0, 3)));
    assert!(!edge_names_child(&branch, &edge(), &chunk(SESSION, 3)));
    assert!(!edge_names_child(
        &branch,
        &edge(),
        &reuse_claim(SESSION, 3, CHUNK, DIGEST)
    ));
}

/// A chunk is judged on what its declaration's frame says: a frame that was
/// read contradicts the chunk whatever the walk left undecided of the
/// declaration, and agreement leaves the chunk as undecided as its declaration.
#[test]
fn a_chunk_is_its_declared_sessions_occurrence_whatever_its_declaration_is_undecided_of() {
    let chunk_outcome = |declared_bytes: u64, declared: Option<Outcome>| {
        let declaration = BlobFact::Declaration {
            store: [1; 16],
            session: SESSION,
            frame_digest: [3; 32],
            object: [4; 16],
            scope: [4; 32],
            chunk_size: CHUNK as u32,
            total: declared_bytes,
            max_checkpoint_sequence: 9,
        };
        let mut rows = vec![selected([5; 24], chunk(SESSION, 0), Outcome::Intact)];
        if let Some(outcome) = declared {
            rows.push(selected([6; 24], declaration, outcome));
        }
        let found = RowIndex::new(&rows, &Coverage::Complete);
        validate_chunks(&mut rows, &found);
        rows[0].outcome.clone()
    };
    let unavailable = Outcome::Unknown(Unknown::WalCoverageUnavailable);
    let mismatch = damage(Cause::ScopeMismatch);
    assert_eq!(chunk_outcome(CHUNK, Some(Outcome::Intact)), Outcome::Intact);
    assert_eq!(chunk_outcome(CHUNK, Some(unavailable.clone())), unavailable);
    assert_eq!(chunk_outcome(CHUNK - 1, Some(Outcome::Intact)), mismatch);
    assert_eq!(chunk_outcome(CHUNK - 1, Some(unavailable)), mismatch);
    assert_eq!(
        chunk_outcome(CHUNK, Some(damage(Cause::Truncation))),
        mismatch
    );
    assert_eq!(chunk_outcome(CHUNK, None), mismatch);
}
