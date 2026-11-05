//! The newest published generation of one partition at or below a coordinate.

use {
    super::OutputPartitionIndex,
    crate::domain_computation::primary_graph::vocabulary::output_lineage::semantic_source::SemanticSource,
    crate::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate,
};

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

mod court_doors_for_test;
