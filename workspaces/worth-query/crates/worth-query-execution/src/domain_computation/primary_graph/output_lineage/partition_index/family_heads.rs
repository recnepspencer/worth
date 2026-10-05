//! The head each partition of one output source offers a reader.

use super::*;

impl WorthQueryApplicationOutputLineage {
    /// The latest published output of every partition at or below
    /// `coordinate` in its own occurrence. A prepared vacancy is passed, and
    /// every visited locator is one unit of work; a partition with no locator
    /// at or below the coordinate still costs its lookup.
    #[allow(clippy::type_complexity)]
    pub(in crate::domain_computation::primary_graph::output_lineage) fn family_partition_heads_budgeted(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        maximum_work: usize,
    ) -> Result<(Vec<(Option<[u8; 32]>, &RecordedOutput)>, usize), ()> {
        if maximum_work == 0 {
            return Err(());
        }
        let mut heads = Vec::new();
        let mut work = 0usize;
        let charge = |work: &mut usize| {
            *work = work.checked_add(1).ok_or(())?;
            (*work <= maximum_work).then_some(()).ok_or(())
        };
        if let Some(partitions) = self
            .partition_index
            .slots
            .get(source)
            .and_then(|occurrences| occurrences.get(&coordinate.occurrence))
        {
            for (partition, generations) in partitions {
                let mut visited = false;
                for (generation, slot) in generations.range(..=coordinate.generation).rev() {
                    visited = true;
                    charge(&mut work)?;
                    let Some(slot) = slot.get() else { continue };
                    heads.push((
                        *partition,
                        self.recorded_at_partition_slot(
                            source,
                            coordinate.occurrence,
                            *generation,
                            *partition,
                            *slot,
                        ),
                    ));
                    break;
                }
                if !visited {
                    charge(&mut work)?;
                }
            }
        }
        Ok((heads, work.max(1)))
    }
}
