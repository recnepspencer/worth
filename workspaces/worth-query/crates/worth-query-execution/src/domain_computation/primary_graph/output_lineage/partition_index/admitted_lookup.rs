//! The selected partition locator charges the caller's cumulative read admission.

use std::sync::{Arc, OnceLock};

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::ProductBranchIncarnation;

use super::{OutputPartitionIndex, ProductCoordinate, RecordedOutput, SemanticSource};
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission, prepared_slot::tree_work,
    WorthQueryApplicationOutputLineage,
};

fn charge_tree<K>(
    admission: &mut InvalidationEditAdmission,
    entries: usize,
) -> Result<(), CompanionPreflightStop> {
    let work = tree_work::<K>(entries).ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    admission.charge_external_work(work)
}

impl OutputPartitionIndex {
    /// A vacant unpublished generation is skipped; every searched level and
    /// visited generation is charged before the corresponding read.
    pub(in crate::domain_computation::primary_graph::output_lineage) fn latest_admitted(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<(u64, usize)>, CompanionPreflightStop> {
        charge_tree::<SemanticSource>(admission, self.slots.len())?;
        let Some(occurrences) = self.slots.get(source) else {
            return Ok(None);
        };
        charge_tree::<ProductBranchIncarnation>(admission, occurrences.len())?;
        let Some(partitions) = occurrences.get(&coordinate.occurrence) else {
            return Ok(None);
        };
        charge_tree::<[u8; 32]>(admission, partitions.len())?;
        let Some(generations) = partitions.get(&partition) else {
            return Ok(None);
        };
        charge_tree::<u64>(admission, generations.len())?;
        for (generation, slot) in generations.range(..=coordinate.generation).rev() {
            admission.charge_external_work(1)?;
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Arc<OnceLock<RecordedOutput>>, CompanionPreflightStop> {
        charge_tree::<SemanticSource>(admission, self.by_source.len())?;
        let occurrences = self
            .by_source
            .get(source)
            .expect("a partition locator retains its output source");
        charge_tree::<ProductBranchIncarnation>(admission, occurrences.len())?;
        let history = occurrences
            .get(&occurrence)
            .expect("a partition locator retains its occurrence");
        charge_tree::<u64>(admission, history.len())?;
        let records = history
            .get(&generation)
            .expect("a partition locator retains its generation");
        admission.charge_external_work(2)?;
        let cell = records
            .get(slot)
            .expect("a partition locator retains its record slot");
        let recorded = cell
            .get()
            .expect("a selected partition locator references a published record");
        admission.charge_external_work(1)?;
        assert_eq!(
            recorded.source_partition_identity,
            Some(partition),
            "a selected partition locator retains its semantic partition"
        );
        admission.charge_external_work(1)?;
        Ok(Arc::clone(cell))
    }
}
