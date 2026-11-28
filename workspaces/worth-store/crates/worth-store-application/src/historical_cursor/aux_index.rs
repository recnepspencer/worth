//! Disposable auxiliary physical index for historical basis cursor lookups.
//!
//! Physical indexes are strictly accelerators: discarding them at any moment
//! never breaks a resumed [`worth_runtime_world::facade::RetainedBasisToken`],
//! never skips rows, and never duplicates rows because canonical chunk reads
//! honor [`CanonicalCursorKey`] strictly in canonical logical key order.

use std::collections::BTreeMap;
use std::sync::RwLock;

use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::CanonicalCursorKey;

use super::chunk_format::SealedChunkDescriptor;

#[derive(Debug, Clone)]
struct BasisKeyIndex {
    /// Sorted `(max_key, chunk_index)` entries accelerating start-chunk lookup.
    chunk_bounds: Vec<(CanonicalCursorKey, usize)>,
}

/// Optional disposable auxiliary physical index registry.
#[derive(Debug, Default)]
pub struct AuxiliaryIndexRegistry {
    indexes: RwLock<BTreeMap<(CommitId, [u8; 32]), BasisKeyIndex>>,
}

impl AuxiliaryIndexRegistry {
    pub fn build_for_basis(
        &self,
        commit_id: CommitId,
        root_semantic_commitment: [u8; 32],
        chunks: &[SealedChunkDescriptor],
    ) {
        let chunk_bounds = chunks
            .iter()
            .enumerate()
            .map(|(index, chunk)| (chunk.max_key.clone(), index))
            .collect();
        let mut guard = self
            .indexes
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.insert(
            (commit_id, root_semantic_commitment),
            BasisKeyIndex { chunk_bounds },
        );
    }

    /// Returns the first chunk index that could contain keys strictly greater
    /// than `after`, if an auxiliary physical index is currently built.
    pub fn first_chunk_after(
        &self,
        commit_id: CommitId,
        root_semantic_commitment: [u8; 32],
        after: &CanonicalCursorKey,
    ) -> Option<usize> {
        let guard = self
            .indexes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let index = guard.get(&(commit_id, root_semantic_commitment))?;
        let pos = index
            .chunk_bounds
            .partition_point(|(max_key, _)| max_key <= after);
        Some(
            index
                .chunk_bounds
                .get(pos)
                .map_or(index.chunk_bounds.len(), |(_, chunk_idx)| *chunk_idx),
        )
    }

    /// Discards ALL auxiliary physical indexes immediately.
    pub fn discard_all(&self) -> usize {
        let mut guard = self
            .indexes
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count = guard.len();
        guard.clear();
        count
    }

    pub fn index_count(&self) -> usize {
        self.indexes
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}
