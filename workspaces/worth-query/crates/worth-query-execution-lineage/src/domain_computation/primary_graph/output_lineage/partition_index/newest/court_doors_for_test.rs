//! Sealed test doors over this module's private members.
use super::*;

impl OutputPartitionIndex {
    #[doc(hidden)]
    pub fn latest_for_test(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
    ) -> Option<(u64, usize)> {
        Self::latest(self, source, coordinate, partition)
    }
}
