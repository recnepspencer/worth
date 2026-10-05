//! Prepared replacement of one derived partition locator under lineage custody.

use super::OutputPartitionIndex;
use crate::domain_computation::primary_graph::{
    output_lineage::{
        invalidation::InvalidationEditAdmission,
        prepared_slot::{arc_bytes, denial, tree_work},
        retained_capacity::RetainedLineageCapacity,
        ProductCoordinate, SemanticSource,
    },
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
};
use std::sync::{Arc, OnceLock};
use worth_runtime_world::facade::ProductBranchIncarnation;

/// An absent address has only an empty hidden vacancy. An existing address
/// remains unchanged until the source installation succeeds. The caller holds
/// the lineage guard from preparation through publication or inverse removal.
pub(in crate::domain_computation::primary_graph::output_lineage) struct PreparedStablePartitionLocator
{
    cell: Arc<OnceLock<usize>>,
    inserted_vacancy: bool,
}

impl OutputPartitionIndex {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn prepare_stable_locator(
        &mut self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        retained: &mut RetainedLineageCapacity,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedStablePartitionLocator, WorthQueryOutputDemandDenial> {
        let outer = tree_work::<SemanticSource>(self.slots.len()).ok_or_else(work_denial)?;
        admission
            .charge_external_work(outer)
            .map_err(|_| work_denial())?;
        let occurrences = self.slots.get(source);
        let middle =
            tree_work::<ProductBranchIncarnation>(occurrences.map_or(0, |rows| rows.len()))
                .ok_or_else(work_denial)?;
        admission
            .charge_external_work(middle)
            .map_err(|_| work_denial())?;
        let partitions = occurrences.and_then(|rows| rows.get(&coordinate.occurrence));
        let inner = tree_work::<Option<[u8; 32]>>(partitions.map_or(0, |rows| rows.len()))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(inner)
            .map_err(|_| work_denial())?;
        let generations = partitions.and_then(|rows| rows.get(&Some(partition)));
        let leaf =
            tree_work::<u64>(generations.map_or(0, |rows| rows.len())).ok_or_else(work_denial)?;
        admission
            .charge_external_work(leaf)
            .map_err(|_| work_denial())?;
        let existing = generations.and_then(|rows| rows.get(&coordinate.generation));
        if existing.is_some_and(|cell| cell.get().is_none()) {
            return Err(denial(Kind::SchedulingDeferred));
        }
        let inserted_vacancy = existing.is_none();
        // Each row retains a full selected path guard even when sharing the
        // present nodes. Retirement of an earlier row cannot unfund this row.
        let bytes = self
            .retained_path_guard_bytes(source, coordinate, Some(partition))
            .and_then(|bytes| bytes.checked_add(arc_bytes::<OnceLock<usize>>()?))
            .ok_or_else(capacity_denial)?;
        // Prepay the exact known insertion growth, the total success path,
        // and the inverse empty-path removal before mutating any owner table.
        let future_outer = tree_work::<SemanticSource>(
            self.slots
                .len()
                .checked_add(usize::from(occurrences.is_none()))
                .ok_or_else(work_denial)?,
        )
        .ok_or_else(work_denial)?;
        let future_middle = tree_work::<ProductBranchIncarnation>(
            occurrences
                .map_or(0, |rows| rows.len())
                .checked_add(usize::from(partitions.is_none()))
                .ok_or_else(work_denial)?,
        )
        .ok_or_else(work_denial)?;
        let future_inner = tree_work::<Option<[u8; 32]>>(
            partitions
                .map_or(0, |rows| rows.len())
                .checked_add(usize::from(generations.is_none()))
                .ok_or_else(work_denial)?,
        )
        .ok_or_else(work_denial)?;
        let future_leaf = tree_work::<u64>(
            generations
                .map_or(0, |rows| rows.len())
                .checked_add(usize::from(inserted_vacancy))
                .ok_or_else(work_denial)?,
        )
        .ok_or_else(work_denial)?;
        let work = outer
            .checked_add(middle)
            .and_then(|n| n.checked_add(inner))
            .and_then(|n| n.checked_add(leaf))
            .and_then(|n| {
                n.checked_add(
                    future_outer
                        .checked_add(future_middle)?
                        .checked_add(future_inner)?
                        .checked_add(future_leaf)?
                        .checked_mul(2)?,
                )
            })
            .and_then(|n| n.checked_add(2))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(work)
            .map_err(|_| work_denial())?;
        admission
            .admit_read_scratch(bytes)
            .map_err(|_| capacity_denial())?;
        retained.reserve_additional(bytes)?;
        let cell = Arc::new(OnceLock::new());
        if inserted_vacancy {
            assert!(self
                .slots
                .entry(source.clone())
                .or_default()
                .entry(coordinate.occurrence)
                .or_default()
                .entry(Some(partition))
                .or_default()
                .insert(coordinate.generation, Arc::clone(&cell))
                .is_none());
        }
        Ok(PreparedStablePartitionLocator {
            cell,
            inserted_vacancy,
        })
    }

    pub(in crate::domain_computation::primary_graph::output_lineage) fn publish_stable_locator(
        &mut self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        slot: usize,
        prepared: PreparedStablePartitionLocator,
    ) -> Option<Arc<OnceLock<usize>>> {
        assert!(prepared.cell.set(slot).is_ok());
        if prepared.inserted_vacancy {
            return None;
        }
        let installed = self
            .slots
            .get_mut(source)
            .unwrap()
            .get_mut(&coordinate.occurrence)
            .unwrap()
            .get_mut(&Some(partition))
            .unwrap()
            .get_mut(&coordinate.generation)
            .unwrap();
        Some(std::mem::replace(installed, prepared.cell))
    }

    pub(in crate::domain_computation::primary_graph::output_lineage) fn abandon_stable_locator(
        &mut self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: [u8; 32],
        prepared: &PreparedStablePartitionLocator,
    ) {
        if prepared.inserted_vacancy {
            self.remove_vacancy(
                source,
                coordinate.occurrence,
                coordinate.generation,
                Some(partition),
            );
        }
    }
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    denial(Kind::WorkBudgetExceeded)
}
fn capacity_denial() -> WorthQueryOutputDemandDenial {
    denial(Kind::RetentionBudgetExceeded)
}
