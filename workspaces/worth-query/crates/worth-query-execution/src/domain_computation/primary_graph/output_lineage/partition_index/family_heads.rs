//! The head each partition of one output source offers a reader.

use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub(in crate::domain_computation::primary_graph::output_lineage) struct FamilyPublicationHead<'a> {
    pub(in crate::domain_computation::primary_graph::output_lineage) coordinate: ProductCoordinate,
    pub(in crate::domain_computation::primary_graph::output_lineage) slot: usize,
    pub(in crate::domain_computation::primary_graph::output_lineage) recorded: &'a RecordedOutput,
}

impl WorthQueryApplicationOutputLineage {
    /// The latest published output of every partition at or below
    /// `coordinate` in its own occurrence. A prepared vacancy is passed, and
    /// every visited locator is one unit of work; a partition with no locator
    /// at or below the coordinate still costs its lookup.
    #[allow(clippy::type_complexity)]
    pub(in crate::domain_computation::primary_graph::output_lineage) fn family_partition_heads(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
    ) -> Result<Vec<(Option<[u8; 32]>, FamilyPublicationHead<'_>)>, CompanionPreflightStop> {
        let mut heads = Vec::new();
        if let Some(partitions) = self
            .partition_index
            .slots
            .get(source)
            .and_then(|occurrences| occurrences.get(&coordinate.occurrence))
        {
            for (partition, generations) in partitions {
                for (generation, slot) in generations.range(..=coordinate.generation).rev() {
                    execution
                        .consult()
                        .map_err(|stop| CompanionPreflightStop::Execution(stop.into()))?;
                    let Some(slot) = slot.get() else { continue };
                    heads.push((
                        *partition,
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
                                *partition,
                                *slot,
                            ),
                        },
                    ));
                    break;
                }
            }
        }
        Ok(heads)
    }
}
