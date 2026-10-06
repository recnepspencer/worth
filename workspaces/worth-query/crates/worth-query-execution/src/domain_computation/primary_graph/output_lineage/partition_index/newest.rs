//! The newest published generation of one partition at or below a coordinate.

use super::{OutputPartitionIndex, ProductCoordinate, SemanticSource};

impl OutputPartitionIndex {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn latest(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        maximum_work: usize,
    ) -> Result<(Option<(u64, usize)>, usize), ()> {
        let Some(generations) = self.generations(source, coordinate, partition) else {
            return (maximum_work != 0).then_some((None, 1)).ok_or(());
        };
        let mut work = 0usize;
        for (generation, slot) in generations.range(..=coordinate.generation).rev() {
            work = work.checked_add(1).ok_or(())?;
            if work > maximum_work {
                return Err(());
            }
            if let Some(slot) = slot.get() {
                return Ok((Some((*generation, *slot)), work));
            }
        }
        if work == 0 {
            work = 1;
        }
        (work <= maximum_work).then_some((None, work)).ok_or(())
    }

    /// The newest published generation at or below `coordinate`, for owners
    /// whose walk no request meters.
    pub(in crate::domain_computation::primary_graph::output_lineage) fn latest_unbudgeted(
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
