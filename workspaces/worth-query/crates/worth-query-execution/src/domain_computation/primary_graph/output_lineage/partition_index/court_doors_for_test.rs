//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn latest_output_in_partition_for_test(
        &self, source: &SemanticSource, coordinate: ProductCoordinate, partition: [u8; 32]) -> Option<&RecordedOutput> {
        Self::latest_output_in_partition(self, source, coordinate, partition)
    }
}
