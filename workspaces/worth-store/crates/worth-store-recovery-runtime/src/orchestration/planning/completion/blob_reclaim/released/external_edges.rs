//! Independent selected Blob edge audit. Rows are emitted in selected route
//! order, after closure reachability is authenticated, matching the physical
//! source/edge transcript without treating a descriptor digest as authority.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{BlobRecordV1, PersistedRecordIdentity};

#[cfg(test)]
#[path = "external_edges/drop_result_tests.rs"]
mod drop_result_tests;

#[derive(Clone)]
pub(super) struct ExternalBlobFact {
    record: PersistedRecordIdentity,
    frame_sha256: [u8; 32],
    same_session: bool,
    edges: Vec<(u8, PersistedRecordIdentity)>,
}

impl ExternalBlobFact {
    pub(super) fn from_record(
        record: PersistedRecordIdentity,
        frame_sha256: [u8; 32],
        fact: &BlobRecordV1,
        source_session: [u8; 16],
    ) -> Self {
        let same_session = match fact {
            BlobRecordV1::TreeNode(node) => node.occurrence().session() == source_session,
            BlobRecordV1::Chunk(chunk) => chunk.occurrence().session() == source_session,
            BlobRecordV1::ChunkReuseClaim(claim) => claim.destination_session() == source_session,
            BlobRecordV1::ChunkReuseClaimV2(value) => {
                value.claim().destination_session() == source_session
            }
            _ => false,
        };
        let mut edges = Vec::new();
        match fact {
            BlobRecordV1::TreeNode(node) => {
                edges.extend(node.entries().iter().map(|entry| (1, entry.record())));
            }
            BlobRecordV1::GenerationPublished(value) => edges.push((2, value.root_record())),
            BlobRecordV1::ChunkReuseClaim(value) => {
                edges.push((3, value.source_publication()));
                edges.push((4, value.selected_chunk()));
            }
            BlobRecordV1::ChunkReuseClaimV2(value) => {
                edges.push((5, value.claim().selected_chunk()));
            }
            BlobRecordV1::DedupeQuarantine(value) => {
                edges.push((6, value.source_publication()));
                edges.push((7, value.source_chunk()));
                edges.push((8, value.conflicting_chunk()));
            }
            BlobRecordV1::SessionFrontier(value) => edges.push((9, value.last_chunk_record())),
            _ => {}
        }
        Self {
            record,
            frame_sha256,
            same_session,
            edges,
        }
    }
}

pub(super) struct ExternalEdgeAudit {
    pub(super) digest: [u8; 32],
    pub(super) publication_referenced: bool,
    pub(super) protected: BTreeSet<PersistedRecordIdentity>,
}

/// Mirrors the producer's semantic result, not merely its drop ordering hash:
/// no live child may be stranded, and terminality means no owned payload remains.
pub(super) fn validates_drop_result(
    reached: &BTreeSet<PersistedRecordIdentity>,
    facts: &[ExternalBlobFact],
    dropped: &[PersistedRecordIdentity],
    protected: &BTreeSet<PersistedRecordIdentity>,
    terminal: bool,
) -> bool {
    let unconnected = facts.iter().any(|fact| {
        fact.same_session && !reached.contains(&fact.record) && !protected.contains(&fact.record)
    });
    let remaining = facts.iter().any(|fact| {
        fact.same_session
            && reached.contains(&fact.record)
            && dropped.binary_search(&fact.record).is_err()
    });
    !unconnected
        && terminal == !remaining
        && facts.iter().all(|fact| {
            dropped.binary_search(&fact.record).is_err()
                || fact.edges.iter().all(|&(role, child)| {
                    role != 1
                        || facts
                            .binary_search_by_key(&child, |fact| fact.record)
                            .ok()
                            .is_none_or(|index| {
                                !facts[index].same_session || dropped.binary_search(&child).is_ok()
                            })
                })
        })
}

pub(super) fn audit(
    publication: PersistedRecordIdentity,
    reached: &BTreeSet<PersistedRecordIdentity>,
    facts: &[ExternalBlobFact],
) -> Option<ExternalEdgeAudit> {
    if facts
        .windows(2)
        .any(|pair| pair[0].record >= pair[1].record)
    {
        return None;
    }
    let mut digest = Sha256::new();
    digest.update(b"store.physical.released-drop-external-edges.v1");
    let mut source_count = 0_u64;
    let mut edge_count = 0_u64;
    let mut publication_referenced = false;
    let mut protected = BTreeSet::new();
    for source in facts {
        if reached.contains(&source.record) || source.record == publication {
            continue;
        }
        source_count = source_count.checked_add(1)?;
        digest.update([0]);
        write_record(&mut digest, source.record);
        digest.update(source.frame_sha256);
        for &(role, target) in &source.edges {
            edge_count = edge_count.checked_add(1)?;
            let target_reachable = reached.contains(&target);
            let target_same_session = facts
                .binary_search_by_key(&target, |fact| fact.record)
                .ok()
                .is_some_and(|index| facts[index].same_session);
            digest.update([1, role]);
            write_record(&mut digest, source.record);
            write_record(&mut digest, target);
            digest.update([
                u8::from(target == publication),
                u8::from(target_reachable),
                u8::from(target_same_session),
            ]);
            publication_referenced |= target == publication;
            if target_reachable || target_same_session {
                protected.insert(target);
            }
        }
    }
    // Direct incoming edges protect a tree's entire selected subtree. The
    // producer applies the same rule after computing this edge transcript;
    // otherwise a child could be dropped while an external owner still
    // references its parent.
    let mut pending = protected.iter().copied().collect::<Vec<_>>();
    while let Some(record) = pending.pop() {
        let Ok(index) = facts.binary_search_by_key(&record, |fact| fact.record) else {
            continue;
        };
        for &(role, child) in &facts[index].edges {
            if role == 1 && protected.insert(child) {
                if pending.len() >= facts.len() {
                    return None;
                }
                pending.push(child);
            }
        }
    }
    digest.update(source_count.to_le_bytes());
    digest.update(edge_count.to_le_bytes());
    Some(ExternalEdgeAudit {
        digest: digest.finalize().into(),
        publication_referenced,
        protected,
    })
}

fn write_record(digest: &mut Sha256, record: PersistedRecordIdentity) {
    digest.update(record.allocation_epoch());
    digest.update(record.ordinal().to_le_bytes());
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use sha2::{Digest, Sha256};
    use worth_store_physical_format::PersistedRecordIdentity;

    use super::{audit, ExternalBlobFact};

    #[test]
    fn external_audit_binds_source_frame_edge_role_and_reachability() {
        let record = |ordinal| PersistedRecordIdentity::new([1; 16], ordinal).unwrap();
        let reached = BTreeSet::from([record(2)]);
        let facts = [
            ExternalBlobFact {
                record: record(1),
                frame_sha256: [3; 32],
                same_session: false,
                edges: vec![(5, record(2))],
            },
            ExternalBlobFact {
                record: record(2),
                frame_sha256: [4; 32],
                same_session: true,
                edges: Vec::new(),
            },
        ];
        let observed = audit(record(3), &reached, &facts).unwrap();
        assert_eq!(observed.protected, reached);
        assert!(!observed.publication_referenced);
        let mut expected = Sha256::new();
        expected.update(b"store.physical.released-drop-external-edges.v1");
        expected.update([0]);
        expected.update([1; 16]);
        expected.update(1_u64.to_le_bytes());
        expected.update([3; 32]);
        expected.update([1, 5]);
        expected.update([1; 16]);
        expected.update(1_u64.to_le_bytes());
        expected.update([1; 16]);
        expected.update(2_u64.to_le_bytes());
        expected.update([0, 1, 1]);
        expected.update(1_u64.to_le_bytes());
        expected.update(1_u64.to_le_bytes());
        assert_eq!(observed.digest, <[u8; 32]>::from(expected.finalize()));
        let mut altered = facts.clone();
        altered[0].edges[0].0 = 4;
        assert_ne!(
            audit(record(3), &reached, &altered).unwrap().digest,
            observed.digest
        );
    }

    #[test]
    fn external_tree_edge_protects_reached_descendant_even_without_direct_edge() {
        let record = |ordinal| PersistedRecordIdentity::new([2; 16], ordinal).unwrap();
        let reached = BTreeSet::from([record(2), record(3)]);
        let facts = [
            ExternalBlobFact {
                record: record(1),
                frame_sha256: [1; 32],
                same_session: false,
                edges: vec![(2, record(2))],
            },
            ExternalBlobFact {
                record: record(2),
                frame_sha256: [2; 32],
                same_session: true,
                edges: vec![(1, record(3))],
            },
            ExternalBlobFact {
                record: record(3),
                frame_sha256: [3; 32],
                same_session: true,
                edges: Vec::new(),
            },
        ];
        let observed = audit(record(4), &reached, &facts).unwrap();
        assert_eq!(observed.protected, reached);
    }
}
