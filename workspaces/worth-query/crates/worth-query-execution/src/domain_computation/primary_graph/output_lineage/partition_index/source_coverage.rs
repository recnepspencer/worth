//! Exact-partition coverage, including invisible publication reservations.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

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
    ) -> Result<SourcePartitionPublication<'_>, WorthQueryOutputDemandDenial> {
        let Some(generations) = self
            .partition_index
            .generations(source, coordinate, partition)
        else {
            return Ok(SourcePartitionPublication::Vacant);
        };

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
