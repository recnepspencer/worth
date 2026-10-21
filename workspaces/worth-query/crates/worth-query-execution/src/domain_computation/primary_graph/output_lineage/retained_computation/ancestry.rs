//! One prior-selection rule: local record, then exact captured fork horizons.
use super::super::ProductCoordinate;
use super::super::RecordedOutput;
use super::super::SemanticSource;
use super::super::WorthQueryApplicationOutputLineage;
use std::sync::Arc;
use std::sync::OnceLock;
use worth_relational::facade::mvcc::CompanionPreflightStop;
impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn computation_cell_in_ancestry(
        &self,
        source: &SemanticSource,
        mut coordinate: ProductCoordinate,
        partition: [u8; 32],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<Arc<OnceLock<RecordedOutput>>>, CompanionPreflightStop> {
        loop {
            if let Some((generation, slot)) = self
                .partition_index
                .latest_admitted(execution, source, coordinate, partition)?
            {
                return self
                    .recorded_cell_at_partition_slot_admitted(
                        source,
                        coordinate.occurrence,
                        generation,
                        partition,
                        slot,
                    )
                    .map(Some);
            }

            let Some(origin) = self.origins.get(&coordinate.occurrence) else {
                return Ok(None);
            };
            coordinate = *origin;
        }
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph::output_lineage) fn fork_pins_computation(
        &self,
        recorded: &RecordedOutput,
    ) -> bool {
        let coordinate = recorded.settlement_identity.coordinate();
        self.origins.values().any(|origin| {
            origin.occurrence == coordinate.occurrence && origin.generation >= coordinate.generation
        })
    }
}
