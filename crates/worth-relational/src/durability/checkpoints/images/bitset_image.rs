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
