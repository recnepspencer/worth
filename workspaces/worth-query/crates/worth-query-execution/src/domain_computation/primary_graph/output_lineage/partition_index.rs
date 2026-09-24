//! Derived partition locators; the recorded output history remains the sole authority.

use std::collections::{BTreeMap, HashMap, HashSet};

use worth_runtime_world::facade::ProductBranchIncarnation;

use super::{
    ProductCoordinate, RecordedOutput, SemanticSource, WorthQueryApplicationOutputLineage,
};

#[derive(Default)]
pub(super) struct OutputPartitionIndex {
    slots: HashMap<
        SemanticSource,
        HashMap<ProductBranchIncarnation, HashMap<[u8; 32], BTreeMap<u64, usize>>>,
    >,
}

impl OutputPartitionIndex {
    pub(super) fn insert(
        &mut self,
        source: SemanticSource,
        occurrence: ProductBranchIncarnation,
        generation: u64,
        partition: [u8; 32],
        slot: usize,
    ) {
        assert!(
            self.slots
                .entry(source)
                .or_default()
                .entry(occurrence)
                .or_default()
                .entry(partition)
                .or_default()
                .insert(generation, slot)
                .is_none(),
            "one product generation may publish one output partition once"
        );
    }

    pub(super) fn at_generation(
        &self,
        source: &SemanticSource,
        occurrence: ProductBranchIncarnation,
        generation: u64,
        partition: [u8; 32],
    ) -> Option<usize> {
        self.slots
            .get(source)?
            .get(&occurrence)?
            .get(&partition)?
            .get(&generation)
            .copied()
    }

    pub(super) fn latest(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
    ) -> Option<(u64, usize)> {
        self.slots
            .get(source)?
            .get(&coordinate.occurrence)?
            .get(&partition)?
            .range(..=coordinate.generation)
            .next_back()
            .map(|(generation, slot)| (*generation, *slot))
    }

    pub(super) fn retain_occurrences(&mut self, retained: &HashSet<ProductBranchIncarnation>) {
        self.slots.retain(|_, occurrences| {
            occurrences.retain(|occurrence, _| retained.contains(occurrence));
            !occurrences.is_empty()
        });
    }
}

impl WorthQueryApplicationOutputLineage {
    fn recorded_at_partition_slot(
        &self,
        source: &SemanticSource,
        occurrence: ProductBranchIncarnation,
        generation: u64,
        partition: [u8; 32],
        slot: usize,
    ) -> &RecordedOutput {
        let recorded = self
            .by_source
            .get(source)
            .and_then(|occurrences| occurrences.get(&occurrence))
            .and_then(|history| history.get(&generation))
            .and_then(|records| records.get(slot))
            .expect("a partition locator must reference retained output authority");
        assert_eq!(
            recorded.source_partition_identity,
            Some(partition),
            "a partition locator must reference the same semantic partition"
        );
        recorded
    }

    pub(super) fn latest_output_in_partition_budgeted(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        maximum_work: usize,
    ) -> Result<(Option<&RecordedOutput>, usize), ()> {
        if maximum_work == 0 {
            return Err(());
        }
        let recorded = self
            .partition_index
            .latest(source, coordinate, partition)
            .map(|(generation, slot)| {
                self.recorded_at_partition_slot(
                    source,
                    coordinate.occurrence,
                    generation,
                    partition,
                    slot,
                )
            });
        Ok((recorded, 1))
    }

    pub(super) fn matching_output_in_partition_budgeted(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        maximum_work: usize,
        mut matches: impl FnMut(&RecordedOutput) -> bool,
    ) -> Result<(Option<&RecordedOutput>, usize), ()> {
        let mut work = 0_usize;
        if let Some(generations) = self
            .partition_index
            .slots
            .get(source)
            .and_then(|occurrences| occurrences.get(&coordinate.occurrence))
            .and_then(|partitions| partitions.get(&partition))
        {
            for (generation, slot) in generations.range(..=coordinate.generation).rev() {
                work = work.checked_add(1).ok_or(())?;
                if work > maximum_work {
                    return Err(());
                }
                let recorded = self.recorded_at_partition_slot(
                    source,
                    coordinate.occurrence,
                    *generation,
                    partition,
                    *slot,
                );
                if matches(recorded) {
                    return Ok((Some(recorded), work));
                }
            }
        }
        if work == 0 {
            if maximum_work == 0 {
                return Err(());
            }
            work = 1;
        }
        Ok((None, work))
    }

    pub(super) fn matching_legacy_output_budgeted(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        maximum_work: usize,
        mut matches: impl FnMut(&RecordedOutput) -> bool,
    ) -> Result<(Option<&RecordedOutput>, usize), ()> {
        let mut work = 0_usize;
        if let Some(history) = self
            .by_source
            .get(source)
            .and_then(|occurrences| occurrences.get(&coordinate.occurrence))
        {
            for (_, records) in history.range(..=coordinate.generation).rev() {
                for recorded in records {
                    work = work.checked_add(1).ok_or(())?;
                    if work > maximum_work {
                        return Err(());
                    }
                    if matches(recorded) {
                        return Ok((Some(recorded), work));
                    }
                }
            }
        }
        if work == 0 {
            if maximum_work == 0 {
                return Err(());
            }
            work = 1;
        }
        Ok((None, work))
    }
}
