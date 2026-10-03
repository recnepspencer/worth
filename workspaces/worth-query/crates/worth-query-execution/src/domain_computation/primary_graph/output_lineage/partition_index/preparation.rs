//! Selected partition vacancy preparation and retained path custody.

use super::*;
use crate::domain_computation::primary_graph::{
    output_lineage::{
        invalidation::InvalidationEditAdmission,
        prepared_slot::{arc_bytes, denial, tree_insert_bytes, tree_work},
    },
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
};

impl OutputPartitionIndex {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn prepare_cancelled_removal(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: Option<[u8; 32]>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(partition) = partition else {
            return Ok(());
        };
        let mut work = tree_work::<SemanticSource>(self.slots.len())
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        admission
            .charge_external_work(work)
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        let occurrences = self.slots.get(source).expect("cancelled partition source");
        let next = tree_work::<ProductBranchIncarnation>(occurrences.len())
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        admission
            .charge_external_work(next)
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        work = work
            .checked_add(next)
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        let partitions = occurrences
            .get(&coordinate.occurrence)
            .expect("cancelled partition occurrence");
        let next = tree_work::<[u8; 32]>(partitions.len())
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        admission
            .charge_external_work(next)
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        work = work
            .checked_add(next)
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        let generations = partitions.get(&partition).expect("cancelled partition");
        let next =
            tree_work::<u64>(generations.len()).ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        work = work
            .checked_add(next)
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        admission
            .charge_external_work(work)
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::output_lineage) fn retained_path_guard_bytes(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: Option<[u8; 32]>,
    ) -> Option<u64> {
        let partition = partition?;
        let occurrences = self.slots.get(source);
        let partitions = occurrences.and_then(|rows| rows.get(&coordinate.occurrence));
        let generations = partitions.and_then(|rows| rows.get(&partition));
        tree_insert_bytes::<SemanticSource, Occurrences>(self.slots.len())?
            .checked_add(tree_insert_bytes::<ProductBranchIncarnation, Partitions>(
                occurrences.map_or(0, |rows| rows.len()),
            )?)?
            .checked_add(tree_insert_bytes::<[u8; 32], Generations>(
                partitions.map_or(0, |rows| rows.len()),
            )?)?
            .checked_add(tree_insert_bytes::<u64, Arc<OnceLock<usize>>>(
                generations.map_or(0, |rows| rows.len()),
            )?)
    }

    pub(in crate::domain_computation::primary_graph::output_lineage) fn vacancy_preparation_bytes(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        partition: Option<[u8; 32]>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<u64, WorthQueryOutputDemandDenial> {
        let Some(partition) = partition else {
            return Ok(0);
        };
        admission
            .charge_external_work(
                tree_work::<SemanticSource>(self.slots.len())
                    .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
            )
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        let occurrences = self.slots.get(source);
        admission
            .charge_external_work(
                tree_work::<ProductBranchIncarnation>(occurrences.map_or(0, |value| value.len()))
                    .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
            )
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        let partitions = occurrences.and_then(|value| value.get(&coordinate.occurrence));
        admission
            .charge_external_work(
                tree_work::<[u8; 32]>(partitions.map_or(0, |value| value.len()))
                    .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
            )
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        let generations = partitions.and_then(|value| value.get(&partition));
        admission
            .charge_external_work(
                tree_work::<u64>(generations.map_or(0, |value| value.len()))
                    .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
            )
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        if generations.is_some_and(|value| value.contains_key(&coordinate.generation)) {
            return Err(denial(Kind::SchedulingDeferred));
        }
        let insertion_work = tree_work::<SemanticSource>(self.slots.len())
            .and_then(|work| {
                work.checked_add(tree_work::<ProductBranchIncarnation>(
                    occurrences.map_or(0, |value| value.len()),
                )?)
            })
            .and_then(|work| {
                work.checked_add(tree_work::<[u8; 32]>(
                    partitions.map_or(0, |value| value.len()),
                )?)
            })
            .and_then(|work| {
                work.checked_add(tree_work::<u64>(
                    generations.map_or(0, |value| value.len()),
                )?)
            })
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?;
        admission
            .charge_external_work(insertion_work)
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        let mut bytes = tree_insert_bytes::<u64, Arc<OnceLock<usize>>>(
            generations.map_or(0, |value| value.len()),
        )
        .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
        bytes = bytes
            .checked_add(
                arc_bytes::<OnceLock<usize>>()
                    .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
            )
            .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
        if occurrences.is_none() {
            bytes = bytes
                .checked_add(
                    tree_insert_bytes::<SemanticSource, Occurrences>(self.slots.len())
                        .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
                )
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
        }
        if partitions.is_none() {
            bytes = bytes
                .checked_add(
                    tree_insert_bytes::<ProductBranchIncarnation, Partitions>(
                        occurrences.map_or(0, |value| value.len()),
                    )
                    .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
                )
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
        }
        if generations.is_none() {
            bytes = bytes
                .checked_add(
                    tree_insert_bytes::<[u8; 32], Generations>(
                        partitions.map_or(0, |value| value.len()),
                    )
                    .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
                )
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
        }
        Ok(bytes)
    }
}
