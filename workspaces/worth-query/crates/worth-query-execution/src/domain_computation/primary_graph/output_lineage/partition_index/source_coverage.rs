//! Exact-partition coverage, including invisible publication reservations.

use super::*;
use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::InvalidationEditAdmission, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind as Kind,
};

pub(in crate::domain_computation::primary_graph::output_lineage) enum SourcePartitionPublication<
    'lineage,
> {
    Vacant,
    Prepared,
    Published(FamilyPublicationHead<'lineage>),
}

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn source_partition_publication(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<SourcePartitionPublication<'_>, WorthQueryOutputDemandDenial> {
        let work =
            crate::domain_computation::primary_graph::output_lineage::prepared_slot::tree_work::<
                SemanticSource,
            >(self.partition_index.slots.len())
            .ok_or_else(work_stop)?;
        admission
            .charge_external_work(work as u64)
            .map_err(|_| work_stop())?;
        let Some(generations) = self
            .partition_index
            .generations(source, coordinate, partition)
        else {
            return Ok(SourcePartitionPublication::Vacant);
        };
        let work = crate::domain_computation::primary_graph::output_lineage::prepared_slot::tree_work::<u64>(generations.len())
            .ok_or_else(work_stop)?;
        admission
            .charge_external_work(work as u64)
            .map_err(|_| work_stop())?;
        let Some((generation, slot)) = generations.range(..=coordinate.generation).next_back()
        else {
            return Ok(SourcePartitionPublication::Vacant);
        };
        let Some(slot) = slot.get() else {
            return Ok(SourcePartitionPublication::Prepared);
        };
        Ok(SourcePartitionPublication::Published(
            FamilyPublicationHead {
                coordinate: ProductCoordinate {
                    occurrence: coordinate.occurrence,
                    generation: *generation,
                },
                slot: *slot,
                recorded: self.recorded_at_partition_slot(
                    source,
                    coordinate.occurrence,
                    *generation,
                    Some(partition),
                    *slot,
                ),
            },
        ))
    }
}

fn work_stop() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(Kind::WorkBudgetExceeded, "output-source-coverage")
}
