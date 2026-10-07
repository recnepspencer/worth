use worth_store_physical_format::PersistedRecordIdentity;

use super::inventory::{SelectedBlobFact, SelectedReleaseFact};

#[derive(Clone, Copy)]
pub(super) enum ExpectedKind {
    RootTree,
    Tree(u8),
    Chunk,
}

#[derive(Clone, Copy)]
pub(super) struct ExpectedEdge {
    pub(super) parent: Option<PersistedRecordIdentity>,
    pub(super) entry_index: u16,
    pub(super) record: PersistedRecordIdentity,
    pub(super) digest: [u8; 32],
    pub(super) covered_bytes: u64,
    pub(super) start: u64,
    pub(super) kind: ExpectedKind,
}

/// A selected leaf must be the precise publication occurrence, including the
/// short final chunk; an authentic digest at another ordinal is insufficient.
pub(super) fn leaf_occurrence_matches(
    total_bytes: u64,
    chunk_size: u32,
    start: u64,
    covered_bytes: u64,
    ordinal: u64,
) -> bool {
    let chunk_size = u64::from(chunk_size);
    chunk_size != 0
        && start % chunk_size == 0
        && covered_bytes != 0
        && total_bytes
            .checked_sub(start)
            .map(|remaining| remaining.min(chunk_size))
            == Some(covered_bytes)
        && ordinal == start / chunk_size
}

pub(super) fn target_session(fact: &SelectedReleaseFact, session: [u8; 16]) -> bool {
    match fact.blob {
        SelectedBlobFact::Tree { session: value, .. }
        | SelectedBlobFact::Chunk { session: value, .. }
        | SelectedBlobFact::ReuseClaim { session: value, .. } => value == session,
        SelectedBlobFact::Other | SelectedBlobFact::Frontier { .. } => false,
    }
}

pub(super) fn is_exclusive_target(fact: &SelectedReleaseFact, session: [u8; 16]) -> bool {
    fact.reachable && target_session(fact, session) && !fact.protected
}

/// A resume frontier the released session wrote for itself. It is residue of
/// that session and leaves with the release; it is never an outside owner of
/// the chunk it names.
pub(super) fn own_frontier(fact: &SelectedReleaseFact, session: [u8; 16]) -> bool {
    matches!(fact.blob, SelectedBlobFact::Frontier { session: value } if value == session)
}

/// Every selected record the release must drop before its head is terminal.
pub(super) fn released_with_session(fact: &SelectedReleaseFact, session: [u8; 16]) -> bool {
    own_frontier(fact, session) || (fact.reachable && target_session(fact, session))
}

#[cfg(test)]
mod tests {
    use super::leaf_occurrence_matches;

    #[test]
    fn release_leaf_occurrence_rejects_wrong_ordinal_alignment_and_final_length() {
        assert!(leaf_occurrence_matches(273, 256, 0, 256, 0));
        assert!(leaf_occurrence_matches(273, 256, 256, 17, 1));
        assert!(!leaf_occurrence_matches(273, 256, 256, 17, 0));
        assert!(!leaf_occurrence_matches(273, 256, 255, 18, 0));
        assert!(!leaf_occurrence_matches(273, 256, 256, 256, 1));
        assert!(!leaf_occurrence_matches(273, 0, 0, 1, 0));
    }
}
