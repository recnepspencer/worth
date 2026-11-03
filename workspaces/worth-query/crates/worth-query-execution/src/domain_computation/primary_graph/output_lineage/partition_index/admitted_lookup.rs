//! The selected partition locator charges the caller's cumulative read admission.

use std::sync::Arc;
use std::sync::OnceLock;

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::ProductBranchIncarnation;

use super::OutputPartitionIndex;
use super::ProductCoordinate;
use super::RecordedOutput;
use super::SemanticSource;
use crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;

impl OutputPartitionIndex {
    /// A vacant unpublished generation is skipped; every searched level and
    /// visited generation is charged before the corresponding read.
    pub(in crate::domain_computation::primary_graph::output_lineage) fn latest_admitted(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
    ) -> Result<Option<(u64, usize)>, CompanionPreflightStop> {
        execution
            .consult()
            .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;
        let Some(occurrences) = self.slots.get(source) else {
            return Ok(None);
        };
        let Some(partitions) = occurrences.get(&coordinate.occurrence) else {
            return Ok(None);
        };

        let Some(generations) = partitions.get(&Some(partition)) else {
            return Ok(None);
        };
        for (generation, slot) in generations.range(..=coordinate.generation).rev() {
            execution
                .consult()
                .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;

            if let Some(slot) = slot.get() {
                return Ok(Some((*generation, *slot)));
            }
        }
        Ok(None)
    }
}

impl WorthQueryApplicationOutputLineage {
    /// The cell is the retained record owner, including its custody tickets.
    /// Pinning it copies one Arc pointer and never copies the receipt payload.
    pub(in crate::domain_computation::primary_graph::output_lineage) fn recorded_cell_at_partition_slot_admitted(
        &self,
        source: &SemanticSource,
        occurrence: ProductBranchIncarnation,
        generation: u64,
        partition: [u8; 32],
        slot: usize,
    ) -> Result<Arc<OnceLock<RecordedOutput>>, CompanionPreflightStop> {
        let occurrences = self
            .by_source
            .get(source)
            .expect("a partition locator retains its output source");
        let history = occurrences
            .get(&occurrence)
            .expect("a partition locator retains its occurrence");
        let records = history
            .get(&generation)
            .expect("a partition locator retains its generation");

        let cell = records
            .get(slot)
            .expect("a partition locator retains its record slot");
        let recorded = cell
            .get()
            .expect("a selected partition locator references a published record");

        assert_eq!(
            recorded.source_partition_identity,
            Some(partition),
            "a selected partition locator retains its semantic partition"
        );

        Ok(Arc::clone(cell))
    }
}
