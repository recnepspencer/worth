//! Derived partition locators; the recorded output history remains the sole authority.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

mod admitted_lookup;
mod history_retirement;
mod preparation;
mod stable_publication;
pub(super) use stable_publication::PreparedStablePartitionLocator;

use worth_runtime_world::facade::ProductBranchIncarnation;

use super::{
    prepared_slot::CancelledLineageSlot, ProductCoordinate, RecordedOutput, SemanticSource,
    WorthQueryApplicationOutputLineage,
};

type Generations = BTreeMap<u64, Arc<OnceLock<usize>>>;
type Partitions = BTreeMap<[u8; 32], Generations>;
type Occurrences = BTreeMap<ProductBranchIncarnation, Partitions>;

#[derive(Default)]
pub(super) struct OutputPartitionIndex {
    slots: BTreeMap<SemanticSource, Occurrences>,
}

impl OutputPartitionIndex {
    pub(super) fn has_address(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
    ) -> bool {
        self.slots
            .get(source)
            .and_then(|occurrences| occurrences.get(&coordinate.occurrence))
            .and_then(|partitions| partitions.get(&partition))
            .is_some_and(|generations| generations.contains_key(&coordinate.generation))
    }

    pub(super) fn insert_vacancy(
        &mut self,
        source: SemanticSource,
        coordinate: ProductCoordinate,
        partition: Option<[u8; 32]>,
    ) -> Option<Arc<OnceLock<usize>>> {
        let Some(partition) = partition else {
            return None;
        };
        let cell = Arc::new(OnceLock::new());
        assert!(self
            .slots
            .entry(source)
            .or_default()
            .entry(coordinate.occurrence)
            .or_default()
            .entry(partition)
            .or_default()
            .insert(coordinate.generation, Arc::clone(&cell))
            .is_none());
        Some(cell)
    }

    pub(super) fn remove_vacancy(
        &mut self,
        source: &SemanticSource,
        occurrence: ProductBranchIncarnation,
        generation: u64,
        partition: [u8; 32],
    ) {
        let occurrences = self
            .slots
            .get_mut(source)
            .expect("prepared partition source stays retained");
        let partitions = occurrences
            .get_mut(&occurrence)
            .expect("prepared partition occurrence stays retained");
        let generations = partitions
            .get_mut(&partition)
            .expect("prepared partition stays retained");
        assert!(generations
            .remove(&generation)
            .is_some_and(|cell| cell.get().is_none()));
        if generations.is_empty() {
            partitions.remove(&partition);
        }
        if partitions.is_empty() {
            occurrences.remove(&occurrence);
        }
        if occurrences.is_empty() {
            self.slots.remove(source);
        }
    }

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
                .insert(generation, {
                    let cell = Arc::new(OnceLock::new());
                    assert!(cell.set(slot).is_ok());
                    cell
                })
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
            .and_then(|cell| cell.get().copied())
    }

    pub(super) fn latest(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        maximum_work: usize,
    ) -> Result<(Option<(u64, usize)>, usize), ()> {
        let Some(generations) = self
            .slots
            .get(source)
            .and_then(|occurrences| occurrences.get(&coordinate.occurrence))
            .and_then(|partitions| partitions.get(&partition))
        else {
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

    pub(super) fn retain_occurrences(
        &mut self,
        retained: &BTreeSet<ProductBranchIncarnation>,
        cancelled: &Option<Box<CancelledLineageSlot>>,
    ) {
        self.slots.retain(|source, occurrences| {
            occurrences.retain(|occurrence, partitions| {
                if retained.contains(occurrence) {
                    return true;
                }
                partitions.retain(|partition, generations| {
                    generations.retain(|generation, _| {
                        CancelledLineageSlot::contains(
                            cancelled,
                            source,
                            ProductCoordinate {
                                occurrence: *occurrence,
                                generation: *generation,
                            },
                            Some(*partition),
                        )
                    });
                    !generations.is_empty()
                });
                !partitions.is_empty()
            });
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
            .and_then(|cell| cell.get())
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
        let (location, work) =
            self.partition_index
                .latest(source, coordinate, partition, maximum_work)?;
        let recorded = location.map(|(generation, slot)| {
            self.recorded_at_partition_slot(
                source,
                coordinate.occurrence,
                generation,
                partition,
                slot,
            )
        });
        Ok((recorded, work))
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
                let Some(slot) = slot.get() else { continue };
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
                    let Some(recorded) = recorded.get() else {
                        continue;
                    };
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
