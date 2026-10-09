use worth_store_physical_format::{
    BlobRecordDenial, BlobTreeEntryV1, BlobTreeNodeKind, BlobTreeNodeV1, BlobTreeOccurrenceV1,
    PersistedRecordIdentity,
};

// C.11 D5 fixes 4,096 chunk/child entries per node. A full node is about
// 256 KiB, within the 512 KiB frame and charged frontier component ceilings.
const FANOUT: usize = 4096;
const MAX_LEVELS: usize = 7;

pub(in crate::physical_runtime::blob) enum BlobTreeBuildFailure<WriteFailure> {
    Format(BlobRecordDenial),
    Write(WriteFailure),
    TooDeep,
}

pub(in crate::physical_runtime::blob) struct BlobTreeRoot {
    pub(in crate::physical_runtime::blob) record: PersistedRecordIdentity,
    pub(in crate::physical_runtime::blob) frame_digest: [u8; 32],
    pub(in crate::physical_runtime::blob) covered_bytes: u64,
}

/// Keeps at most one bounded frontier per level. Record identities enter the
/// tree only after their C5 append and root publication completed.
pub(in crate::physical_runtime::blob) struct BlobTreeBuilder {
    store: [u8; 16],
    session: [u8; 16],
    levels: Vec<Vec<BlobTreeEntryV1>>,
    next_index: Vec<u64>,
    last_node: Option<BlobTreeRoot>,
}

impl BlobTreeBuilder {
    pub(in crate::physical_runtime::blob) const fn maximum_levels() -> usize {
        MAX_LEVELS
    }

    /// Nodes emitted by ordinary full-chunk pushes, before any finish flush.
    pub(in crate::physical_runtime::blob) fn full_node_count(mut chunks: u64, level: u8) -> u64 {
        if level as usize >= MAX_LEVELS {
            return 0;
        }
        for _ in 0..=level {
            chunks /= FANOUT as u64;
        }
        chunks
    }

    pub(in crate::physical_runtime::blob) fn new(store: [u8; 16], session: [u8; 16]) -> Self {
        Self {
            store,
            session,
            levels: vec![Vec::new()],
            next_index: vec![0],
            last_node: None,
        }
    }

    pub(in crate::physical_runtime::blob) fn push<Failure>(
        &mut self,
        entry: BlobTreeEntryV1,
        write: &mut impl FnMut(BlobTreeNodeV1, u64) -> Result<PersistedRecordIdentity, Failure>,
    ) -> Result<(), BlobTreeBuildFailure<Failure>> {
        self.push_at(0, entry, write)
    }

    pub(in crate::physical_runtime::blob) fn finish<Failure>(
        &mut self,
        write: &mut impl FnMut(BlobTreeNodeV1, u64) -> Result<PersistedRecordIdentity, Failure>,
    ) -> Result<BlobTreeRoot, BlobTreeBuildFailure<Failure>> {
        for level in 0..MAX_LEVELS {
            if level >= self.levels.len() || self.levels[level].is_empty() {
                continue;
            }
            if level > 0
                && self.levels[level].len() == 1
                && self.levels.iter().skip(level + 1).all(Vec::is_empty)
            {
                let entry = self.levels[level][0];
                let last = self
                    .last_node
                    .as_ref()
                    .ok_or(BlobTreeBuildFailure::TooDeep)?;
                if last.record != entry.record() || last.covered_bytes != entry.covered_bytes() {
                    return Err(BlobTreeBuildFailure::TooDeep);
                }
                return Ok(BlobTreeRoot {
                    record: last.record,
                    frame_digest: last.frame_digest,
                    covered_bytes: last.covered_bytes,
                });
            }
            self.flush(level, write)?;
        }
        Err(BlobTreeBuildFailure::TooDeep)
    }

    fn push_at<Failure>(
        &mut self,
        level: usize,
        entry: BlobTreeEntryV1,
        write: &mut impl FnMut(BlobTreeNodeV1, u64) -> Result<PersistedRecordIdentity, Failure>,
    ) -> Result<(), BlobTreeBuildFailure<Failure>> {
        if level >= MAX_LEVELS {
            return Err(BlobTreeBuildFailure::TooDeep);
        }
        if level == self.levels.len() {
            self.levels.push(Vec::new());
            self.next_index.push(0);
        }
        self.levels[level].push(entry);
        if self.levels[level].len() == FANOUT {
            self.flush(level, write)?;
        }
        Ok(())
    }

    fn flush<Failure>(
        &mut self,
        level: usize,
        write: &mut impl FnMut(BlobTreeNodeV1, u64) -> Result<PersistedRecordIdentity, Failure>,
    ) -> Result<(), BlobTreeBuildFailure<Failure>> {
        if level + 1 >= MAX_LEVELS {
            return Err(BlobTreeBuildFailure::TooDeep);
        }
        let index = self.next_index[level];
        let kind = if level == 0 {
            BlobTreeNodeKind::Leaf
        } else {
            BlobTreeNodeKind::Interior
        };
        let occurrence = BlobTreeOccurrenceV1::new(
            self.store,
            self.session,
            kind,
            u8::try_from(level).expect("bounded tree level fits u8"),
            index,
        )
        .map_err(BlobTreeBuildFailure::Format)?;
        let entries = std::mem::take(&mut self.levels[level]);
        let node =
            BlobTreeNodeV1::new(occurrence, entries).map_err(BlobTreeBuildFailure::Format)?;
        let digest = node.canonical_digest();
        let frame_digest = node.frame_digest();
        let covered = node.covered_bytes();
        let ordinal = (level as u64) << 56 | index;
        let record = write(node, ordinal).map_err(BlobTreeBuildFailure::Write)?;
        self.last_node = Some(BlobTreeRoot {
            record,
            frame_digest,
            covered_bytes: covered,
        });
        self.next_index[level] = index.checked_add(1).ok_or(BlobTreeBuildFailure::TooDeep)?;
        let parent =
            BlobTreeEntryV1::new(digest, record, covered).map_err(BlobTreeBuildFailure::Format)?;
        self.push_at(level + 1, parent, write)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([7; 16], ordinal).expect("nonzero identity")
    }

    #[test]
    fn thirty_two_chunks_make_one_durable_leaf_root() {
        let mut tree = BlobTreeBuilder::new([1; 16], [2; 16]);
        let mut written = Vec::new();
        let mut writer = |node: BlobTreeNodeV1, ordinal| {
            written.push((node.occurrence().kind(), node.entries().len()));
            Ok::<_, ()>(record(1_000 + ordinal))
        };
        for ordinal in 0..32 {
            tree.push(
                BlobTreeEntryV1::new([3; 32], record(ordinal + 1), 256 * 1024).unwrap(),
                &mut writer,
            )
            .unwrap_or_else(|_| panic!("push must fit"));
        }
        let root = tree
            .finish(&mut writer)
            .unwrap_or_else(|_| panic!("root must fit"));
        assert_eq!(written, [(BlobTreeNodeKind::Leaf, 32)]);
        assert_eq!(root.covered_bytes, 8 * 1024 * 1024);
        assert_ne!(root.frame_digest, [0; 32]);
    }

    #[test]
    fn four_thousand_ninety_seventh_entry_starts_a_second_leaf() {
        let mut tree = BlobTreeBuilder::new([1; 16], [2; 16]);
        let mut written = Vec::new();
        let mut next_record = 10_000;
        let mut writer = |node: BlobTreeNodeV1, _ordinal| {
            written.push((node.occurrence().kind(), node.entries().len()));
            next_record += 1;
            Ok::<_, ()>(record(next_record))
        };
        for ordinal in 0..4097 {
            tree.push(
                BlobTreeEntryV1::new([3; 32], record(ordinal + 1), 64 * 1024).unwrap(),
                &mut writer,
            )
            .unwrap_or_else(|_| panic!("push must fit"));
        }
        let root = tree
            .finish(&mut writer)
            .unwrap_or_else(|_| panic!("root must fit"));
        assert_eq!(
            written,
            [
                (BlobTreeNodeKind::Leaf, 4096),
                (BlobTreeNodeKind::Leaf, 1),
                (BlobTreeNodeKind::Interior, 2),
            ]
        );
        assert_eq!(root.covered_bytes, 4097 * 64 * 1024);
    }
}
