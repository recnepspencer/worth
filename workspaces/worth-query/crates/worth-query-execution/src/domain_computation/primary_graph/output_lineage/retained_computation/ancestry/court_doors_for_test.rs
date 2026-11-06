//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn computation_cell_in_ancestry_for_test(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<Arc<OnceLock<RecordedOutput>>>, CompanionPreflightStop> {
        Self::computation_cell_in_ancestry(self, source, coordinate, partition, execution)
    }
}
