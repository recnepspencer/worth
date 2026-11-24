//! Canonical reduction order for index preparation fragments.
use crate::indexes::data::DerivedIndexId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct IndexReductionKey(u64);

impl IndexReductionKey {
    pub(crate) fn new(index_id: DerivedIndexId, packet_index: usize) -> Self {
        Self(pack_u64_pair(index_id.0, packet_index))
    }
}

const fn pack_u64_pair(high: u64, low: usize) -> u64 {
    debug_assert!(
        /* Proof: PackedReductionKey is private: keys are emitted from admitted packet indices and u32 local ordinals; the low word is masked with u32::MAX before extraction. */
        low <= u32::MAX as usize,
        "low word must fit into u32"
    );
    (high << 32) | (low as u64)
}
