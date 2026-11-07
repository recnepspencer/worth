use crate::durability::data::DurableBitSet;
use crate::storage::partition::DenseSlotBitSet;

pub(super) fn restore_bitset(image: DurableBitSet) -> DenseSlotBitSet {
    if image.sparse_words.is_empty() {
        DenseSlotBitSet::from_words(image.words)
    } else {
        DenseSlotBitSet::from_sparse_words(image.sparse_words)
    }
}

pub(super) fn restore_bitset_for_request(
    image: DurableBitSet,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<DenseSlotBitSet, crate::execution::RelationalExecutionDenialCause> {
    if image.sparse_words.is_empty() {
        DenseSlotBitSet::from_words_for_request(image.words.into_iter().enumerate(), request)
    } else {
        DenseSlotBitSet::from_words_for_request(
            image
                .sparse_words
                .into_iter()
                .map(|(index, word)| (index as usize, word)),
            request,
        )
    }
}
