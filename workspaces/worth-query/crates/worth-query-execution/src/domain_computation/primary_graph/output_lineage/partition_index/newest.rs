//! The newest published generation of one partition at or below a coordinate.

use super::{OutputPartitionIndex, ProductCoordinate, SemanticSource};

impl OutputPartitionIndex {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn latest(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
    ) -> Option<(u64, usize)> {
        self.generations(source, coordinate, partition)?
            .range(..=coordinate.generation)
            .rev()
            .find_map(|(generation, slot)| slot.get().map(|slot| (*generation, *slot)))
    }
}
