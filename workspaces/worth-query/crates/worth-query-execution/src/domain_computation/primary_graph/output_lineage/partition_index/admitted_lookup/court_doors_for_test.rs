//! Sealed test doors over this module's private members.
use super::*;

impl OutputPartitionIndex {
    #[doc(hidden)]
    pub fn latest_admitted_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
    ) -> Result<Option<(u64, usize)>, CompanionPreflightStop> {
        Self::latest_admitted(self, execution, source, coordinate, partition)
    }
}
