//! Independent ordered-drop transcript over an authenticated selected closure.
//! The manifest is identity-sorted; physical removal order is publication,
//! the session's resume frontiers, leaves, then tree levels. A stage byte is
//! a transcript tag, not a position. A digest never proves exclusivity by
//! itself.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{BlobRecordKind, PersistedRecordIdentity};

#[derive(Clone, Copy)]
pub(super) struct TypedClosureRecord {
    pub(super) record: PersistedRecordIdentity,
    pub(super) kind: BlobRecordKind,
    pub(super) tree_level: Option<u8>,
}

pub(super) fn digest(
    publication: PersistedRecordIdentity,
    dropped: &[PersistedRecordIdentity],
    closure: &[TypedClosureRecord],
) -> Option<[u8; 32]> {
    if dropped.is_empty() || dropped.windows(2).any(|pair| pair[0] >= pair[1]) {
        return None;
    }
    if closure
        .windows(2)
        .any(|pair| pair[0].record >= pair[1].record)
    {
        return None;
    }
    let mut frontiers = Vec::new();
    let mut leaves = Vec::new();
    let mut trees = Vec::new();
    let mut publication_dropped = false;
    for &record in dropped {
        if record == publication {
            publication_dropped = true;
            continue;
        }
        let fact = closure
            .binary_search_by_key(&record, |fact| fact.record)
            .ok()
            .and_then(|index| closure.get(index))?;
        match (fact.kind, fact.tree_level) {
            (BlobRecordKind::Chunk | BlobRecordKind::ChunkReuseClaimV2, None) => {
                leaves.push(record);
            }
            (BlobRecordKind::TreeNode, Some(level)) => trees.push((level, record)),
            (BlobRecordKind::SessionFrontier, None) => frontiers.push(record),
            _ => return None,
        }
    }
    trees.sort_unstable();
    let mut transcript = Sha256::new();
    transcript.update(b"store.physical.released-drop-postorder.v1");
    write_record(&mut transcript, publication);
    if publication_dropped {
        transcript.update([0]);
        write_record(&mut transcript, publication);
    }
    for record in frontiers {
        transcript.update([3]);
        write_record(&mut transcript, record);
    }
    for record in leaves {
        transcript.update([1]);
        write_record(&mut transcript, record);
    }
    for (_, record) in trees {
        transcript.update([2]);
        write_record(&mut transcript, record);
    }
    transcript.update((dropped.len() as u64).to_le_bytes());
    Some(transcript.finalize().into())
}

fn write_record(transcript: &mut Sha256, record: PersistedRecordIdentity) {
    transcript.update(record.allocation_epoch());
    transcript.update(record.ordinal().to_le_bytes());
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use worth_store_physical_format::{BlobRecordKind, PersistedRecordIdentity};

    use super::{digest, TypedClosureRecord};

    #[test]
    fn physical_postorder_is_not_manifest_identity_order() {
        let record = |ordinal| PersistedRecordIdentity::new([7; 16], ordinal).unwrap();
        let closure = [
            TypedClosureRecord {
                record: record(1),
                kind: BlobRecordKind::TreeNode,
                tree_level: Some(1),
            },
            TypedClosureRecord {
                record: record(2),
                kind: BlobRecordKind::Chunk,
                tree_level: None,
            },
            TypedClosureRecord {
                record: record(4),
                kind: BlobRecordKind::TreeNode,
                tree_level: Some(0),
            },
        ];
        let dropped = [record(1), record(2), record(3), record(4)];
        let observed = digest(record(3), &dropped, &closure).unwrap();
        let mut expected = Sha256::new();
        expected.update(b"store.physical.released-drop-postorder.v1");
        expected.update([7; 16]);
        expected.update(3_u64.to_le_bytes());
        for (stage, ordinal) in [(0_u8, 3_u64), (1, 2), (2, 4), (2, 1)] {
            expected.update([stage]);
            expected.update([7; 16]);
            expected.update(ordinal.to_le_bytes());
        }
        expected.update(4_u64.to_le_bytes());
        assert_eq!(observed, <[u8; 32]>::from(expected.finalize()));
        assert!(digest(record(3), &[record(2), record(2)], &closure).is_none());
        assert!(digest(record(3), &[record(2), record(5)], &closure).is_none());
    }

    #[test]
    fn resume_frontiers_leave_after_the_publication_and_before_every_leaf() {
        let record = |ordinal| PersistedRecordIdentity::new([7; 16], ordinal).unwrap();
        let closure = [
            TypedClosureRecord {
                record: record(1),
                kind: BlobRecordKind::Chunk,
                tree_level: None,
            },
            TypedClosureRecord {
                record: record(2),
                kind: BlobRecordKind::TreeNode,
                tree_level: Some(0),
            },
            TypedClosureRecord {
                record: record(4),
                kind: BlobRecordKind::SessionFrontier,
                tree_level: None,
            },
            TypedClosureRecord {
                record: record(5),
                kind: BlobRecordKind::SessionDeclared,
                tree_level: None,
            },
            TypedClosureRecord {
                record: record(6),
                kind: BlobRecordKind::SessionFrontier,
                tree_level: None,
            },
        ];
        let dropped = [record(1), record(2), record(3), record(4), record(6)];
        let observed = digest(record(3), &dropped, &closure).unwrap();
        let mut expected = Sha256::new();
        expected.update(b"store.physical.released-drop-postorder.v1");
        expected.update([7; 16]);
        expected.update(3_u64.to_le_bytes());
        // Both frontiers, in record order, before the one leaf.
        for (stage, ordinal) in [(0_u8, 3_u64), (3, 4), (3, 6), (1, 1), (2, 2)] {
            expected.update([stage]);
            expected.update([7; 16]);
            expected.update(ordinal.to_le_bytes());
        }
        expected.update(5_u64.to_le_bytes());
        assert_eq!(observed, <[u8; 32]>::from(expected.finalize()));
        // The declaration is never part of a released drop.
        assert!(digest(record(3), &[record(4), record(5)], &closure).is_none());
    }
}
