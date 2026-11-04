//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn latest_output_in_partition_for_test(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
    ) -> Option<&RecordedOutput> {
        Self::latest_output_in_partition(self, source, coordinate, partition)
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn matching_legacy_output_budgeted_for_test(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        matches: impl FnMut(&RecordedOutput) -> bool,
    ) -> Result<
        (Option<&RecordedOutput>, usize),
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        Self::matching_legacy_output_budgeted(self, source, coordinate, execution, matches)
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn matching_output_in_partition_budgeted_for_test(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        matches: impl FnMut(&RecordedOutput) -> bool,
    ) -> Result<
        (Option<&RecordedOutput>, usize),
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        Self::matching_output_in_partition_budgeted(
            self, source, coordinate, partition, execution, matches,
        )
    }
}
