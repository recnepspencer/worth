//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::semantic_source::SemanticSource;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate;

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
