use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::DurableBitSet;
use worth_relational_storage::internal_port::storage::partition::bitsets::DenseSlotBitSet;

pub(super) fn restore_bitset(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    image: DurableBitSet,
) -> DenseSlotBitSet {
    if image.sparse_words.is_empty() {
        DenseSlotBitSet::from_words(lane, image.words)
    } else {
        DenseSlotBitSet::from_sparse_words(lane, image.sparse_words)
    }
}

pub(super) fn restore_bitset_for_request(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    image: DurableBitSet,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<DenseSlotBitSet, contracts::execution::denial_cause::RelationalExecutionDenialCause> {
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
