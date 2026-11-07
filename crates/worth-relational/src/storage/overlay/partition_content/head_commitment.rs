use super::*;
use crate::storage::substrate::HistoryAppendFrontier;

/// History Merkle right spines carry enough proof to check current content
/// against the original region leaf and to append without reading old values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct PartitionHistoryFrontiers {
    pub(super) entities: Vec<(u64, HistoryAppendFrontier)>,
    pub(super) relations: Vec<(u64, HistoryAppendFrontier)>,
}

impl PartitionHistoryFrontiers {
    pub(crate) fn into_arenas(
        self,
    ) -> (
        Vec<(u64, HistoryAppendFrontier)>,
        Vec<(u64, HistoryAppendFrontier)>,
    ) {
        (self.entities, self.relations)
    }
}

impl PartitionContentCommitment {
    pub(crate) fn history_frontiers(
        &self,
        partition: &PartitionState,
    ) -> Option<PartitionHistoryFrontiers> {
        Some(PartitionHistoryFrontiers {
            entities: self.entities.history_frontiers(&partition.entity_arena)?,
            relations: self
                .relations
                .history_frontiers(&partition.relation_arena)?,
        })
    }

    /// The caller compares the resulting digest with its committed leaf.
    pub(crate) fn capture_head_for_request(
        partition: &PartitionState,
        symbols: &StringInterner,
        frontiers: &PartitionHistoryFrontiers,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, PartitionContentDigestError> {
        full::capture_with_frontiers(partition, symbols, frontiers, Some(request))
            .map(|(commitment, _)| commitment)
    }
}

impl PartitionContentCommitment {
    pub(crate) fn borrowed_history_frontiers<'a>(
        &'a self,
        partition: &'a PartitionState,
    ) -> impl serde::Serialize + 'a {
        #[derive(serde::Serialize)]
        struct View<E: serde::Serialize, R: serde::Serialize> {
            entities: E,
            relations: R,
        }
        View {
            entities: self.entities.frontier_view(&partition.entity_arena),
            relations: self.relations.frontier_view(&partition.relation_arena),
        }
    }
}

pub(super) fn history_from_frontier<K: crate::storage::substrate::RecordKind>(
    arena: &crate::storage::substrate::RecordArena<K>,
    slot: usize,
    ordinal: usize,
    frontiers: &[(u64, HistoryAppendFrontier)],
    request: full::Request<'_, '_, '_>,
) -> Result<(digest_index::DigestIndex, u64), PartitionContentDigestError>
where
    K::Meta: serde::Serialize,
{
    let invalid = || PartitionContentDigestError::InvalidHistoryCoverage;
    let (recorded_slot, frontier) = frontiers.get(ordinal).ok_or_else(invalid)?;
    if *recorded_slot != slot as u64 {
        return Err(invalid());
    }
    let history = digest_index::DigestIndex::from_append_frontier_using(frontier, request)?;
    let metadata = arena.metadata_history_at(slot).ok_or_else(invalid)?;
    if metadata.len() != usize::from(history.next_history_index().ok_or_else(invalid)? > 0) {
        return Err(invalid());
    }
    let last = metadata
        .get(0)
        .map(|value| hash_value(b"metadata", value))
        .transpose()?;
    let count = frontier
        .siblings
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(4))
        .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
    let bytes =
        std::alloc::Layout::array::<crate::storage::substrate::HistoryFrontierSibling>(count)
            .ok()
            .and_then(|layout| u64::try_from(layout.size()).ok())
            .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
    let _scratch = request
        .map(|request| {
            request
                .reserve_retained_memory(bytes)
                .map_err(|cause| PartitionContentDigestError::Request(cause.into()))
        })
        .transpose()?
        .flatten();
    history.append_frontier(last).ok_or_else(invalid)?;
    Ok((history, metadata.len() as u64))
}
