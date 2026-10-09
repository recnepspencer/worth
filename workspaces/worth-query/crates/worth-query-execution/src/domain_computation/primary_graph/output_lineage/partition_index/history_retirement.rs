//! Generations of one occurrence's history that no retained reader selects.
//!
//! A reader selects, per partition, the newest record at or below its
//! coordinate. The invalidation window keeps an occurrence's newest
//! generations; a reader older than the window's oldest retained position
//! verifies in full, as an invalidation reader older than `root.past` does.
//! A generation is freed once, for each record in it, a newer record of that
//! partition is published at or below the oldest retained position, and
//! nothing pins it: no handle on its cell, settlement or locator, and no fork
//! that branched while it was selected. Every retained reader selects that
//! newer record or one after it, so its selection is unchanged. An older
//! generation that a live demand still pins does not keep the unpinned ones
//! behind it: history is bounded by what is held, not by what was published.

use std::sync::Arc;

use worth_runtime_world::facade::ProductBranchIncarnation;

use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission,
    prepared_slot::{denial, tree_work},
    SemanticSource, WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
};

impl WorthQueryApplicationOutputLineage {
    /// Free the generations of `occurrence`'s history under `source` that no
    /// retained reader selects any longer.
    pub(in crate::domain_computation::primary_graph::output_lineage) fn retire_unselected_generations(
        &mut self,
        source: &SemanticSource,
        occurrence: ProductBranchIncarnation,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(window) = self.retention.history_positions() else {
            return Ok(());
        };
        let Some(history) = self
            .by_source
            .get(source)
            .and_then(|occurrences| occurrences.get(&occurrence))
        else {
            return Ok(());
        };
        let history_work = tree_work::<u64>(history.len()).ok_or_else(work_denial)?;
        charge_navigation(admission, 1, history_work)?;
        // nth steps over retained entries; every stepped entry is logical work.
        charge_history_window(admission, window.get().min(history.len()))?;
        let Some(&oldest_retained) = history.keys().rev().nth(window.get() - 1) else {
            return Ok(());
        };
        let mut next = history.keys().next().copied();
        while let Some(generation) = next.filter(|generation| *generation < oldest_retained) {
            charge_navigation(admission, 1, history_work)?;
            let retire =
                !self.selectable(source, occurrence, generation, oldest_retained, admission)?;
            if retire {
                self.retire_generation(source, occurrence, generation, admission)?;
            }
            next = self.by_source[source][&occurrence]
                .range(generation + 1..)
                .next()
                .map(|(generation, _)| *generation);
        }
        Ok(())
    }

    /// Whether a retained reader may still select some record of `generation`.
    fn selectable(
        &self,
        source: &SemanticSource,
        occurrence: ProductBranchIncarnation,
        generation: u64,
        oldest_retained: u64,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let partitions = self
            .partition_index
            .slots
            .get(source)
            .and_then(|occurrences| occurrences.get(&occurrence));
        for cell in &self.by_source[source][&occurrence][&generation] {
            charge(
                admission,
                u64::try_from(self.origins.len())
                    .ok()
                    .and_then(|forks| forks.checked_add(2))
                    .ok_or_else(work_denial)?,
            )?;
            let Some(recorded) = cell.get().filter(|_| Arc::strong_count(cell) == 1) else {
                return Ok(true);
            };
            let Some(partition) = recorded.source_partition_identity else {
                return Ok(true);
            };
            let Some(generations) =
                partitions.and_then(|partitions| partitions.get(&Some(partition)))
            else {
                return Ok(true);
            };
            if generations
                .get(&generation)
                .is_none_or(|locator| Arc::strong_count(locator) != 1)
            {
                return Ok(true);
            }
            // The newer record that readers at the oldest retained position
            // select in its place.
            let mut successor = None;
            for (newer, locator) in generations.range(generation + 1..=oldest_retained) {
                charge(admission, 1)?;
                if locator.get().is_some() {
                    successor = Some(*newer);
                    break;
                }
            }
            let Some(successor) = successor else {
                return Ok(true);
            };
            if self.origins.values().any(|fork| {
                fork.occurrence == occurrence && (generation..successor).contains(&fork.generation)
            }) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn retire_generation(
        &mut self,
        source: &SemanticSource,
        occurrence: ProductBranchIncarnation,
        generation: u64,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let history = self
            .by_source
            .get_mut(source)
            .and_then(|occurrences| occurrences.get_mut(&occurrence))
            .expect("a retired generation's history is retained");
        let removal = tree_work::<u64>(history.len()).ok_or_else(work_denial)?;
        let records = history
            .get(&generation)
            .expect("a retired generation is retained");
        let operations = u64::try_from(records.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .ok_or_else(work_denial)?;
        charge_navigation(
            admission,
            operations,
            operations.checked_mul(removal).ok_or_else(work_denial)?,
        )?;
        let retired = history
            .remove(&generation)
            .expect("a retired generation is retained");
        let partitions = self
            .partition_index
            .slots
            .get_mut(source)
            .and_then(|occurrences| occurrences.get_mut(&occurrence))
            .expect("a retired generation's partitions are indexed");
        for cell in &retired {
            let partition = cell
                .get()
                .and_then(|recorded| recorded.source_partition_identity)
                .expect("a retired record is a published partition record");
            let generations = partitions
                .get_mut(&Some(partition))
                .expect("a retired record's partition is indexed");
            assert!(generations.remove(&generation).is_some());
            assert!(
                !generations.is_empty(),
                "a newer record of the partition replaces the retired one"
            );
        }
        // The records' custody refunds as they drop.
        drop(retired);
        Ok(())
    }
}

fn charge(
    admission: &mut InvalidationEditAdmission,
    work: u64,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(work)
        .map_err(|_| work_denial())
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    denial(Kind::WorkBudgetExceeded)
}

fn charge_navigation(
    admission: &mut InvalidationEditAdmission,
    operations: u64,
    navigation: u64,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_ordered_operations(operations, navigation)
        .map_err(|_| work_denial())
}

fn charge_history_window(
    admission: &mut InvalidationEditAdmission,
    entries: usize,
) -> Result<(), WorthQueryOutputDemandDenial> {
    charge(
        admission,
        u64::try_from(entries).map_err(|_| work_denial())?,
    )
}
#[cfg(test)]
mod window_work {
    use super::*;
    #[test]
    fn a_linear_history_window_spends_one_logical_visit_per_entry() {
        let visits =
            worth_relational::facade::indexes::SelectedIndexReadWork::MAXIMUM_ORDERED_DESCENT_WORK
                + 1;
        let mut admission = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: visits,
                maximum_preparation_bytes: 0,
            },
        );
        charge_history_window(&mut admission, visits as usize).unwrap();
        assert_eq!(admission.charged_work(), visits);
        assert_eq!(admission.charged_navigation(), 0);
    }
}
