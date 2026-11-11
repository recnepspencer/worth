use crate::durability::data::DurableBitSet;
use crate::storage::partition::DenseSlotBitSet;

pub(super) fn restore_bitset(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    image: DurableBitSet,
) -> DenseSlotBitSet {
    if image.sparse_words.is_empty() {
        DenseSlotBitSet::from_words(lane, image.words)
    } else {
        DenseSlotBitSet::from_sparse_words(lane, image.sparse_words)
    }
}

pub(super) fn restore_bitset_for_request(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    image: DurableBitSet,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<DenseSlotBitSet, crate::execution::RelationalExecutionDenialCause> {
    if image.sparse_words.is_empty() {
        DenseSlotBitSet::from_words_for_request(lane, image.words.into_iter().enumerate(), request)
    } else {
        DenseSlotBitSet::from_words_for_request(
            lane,
            image
                .sparse_words
                .into_iter()
                .map(|(index, word)| (index as usize, word)),
            request,
        )
    }
}
