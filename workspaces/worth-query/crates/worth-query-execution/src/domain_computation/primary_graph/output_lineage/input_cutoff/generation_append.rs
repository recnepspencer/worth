//! Selected generation storage prepared before stable source publication.

use super::super::{
    invalidation::InvalidationEditAdmission,
    prepared_slot::{denial, tree_insert_bytes, tree_work},
    retained_capacity::RetainedLineageCapacity,
    ProductCoordinate, RecordedGeneration, SemanticSource, WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
};
use std::collections::BTreeMap;
use worth_runtime_world::facade::ProductBranchIncarnation;

pub(super) struct PreparedGenerationAppend {
    inserted_generation: bool,
    replacement: Option<RecordedGeneration>,
}

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph::output_lineage::input_cutoff) fn prepare_stable_generation(
        &mut self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        expected_count: usize,
        retained: &mut RetainedLineageCapacity,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedGenerationAppend, WorthQueryOutputDemandDenial> {
        let outer = tree_work::<SemanticSource>(self.by_source.len()).ok_or_else(work_denial)?;
        admission
            .charge_ordered_operations(1, outer)
            .map_err(|_| work_denial())?;
        let occurrences = self.by_source.get(source);
        let middle =
            tree_work::<ProductBranchIncarnation>(occurrences.map_or(0, |rows| rows.len()))
                .ok_or_else(work_denial)?;
        admission
            .charge_ordered_operations(1, middle)
            .map_err(|_| work_denial())?;
        let history = occurrences.and_then(|rows| rows.get(&coordinate.occurrence));
        let inner =
            tree_work::<u64>(history.map_or(0, |rows| rows.len())).ok_or_else(work_denial)?;
        admission
            .charge_ordered_operations(1, inner)
            .map_err(|_| work_denial())?;
        let generation = history.and_then(|rows| rows.get(&coordinate.generation));
        if generation.map_or(0, Vec::len) != expected_count {
            return Err(denial(Kind::PublicationStale));
        }
        let inserted_generation = generation.is_none();
        let grow = generation.is_none_or(|rows| rows.len() == rows.capacity());
        let capacity = if grow {
            expected_count
                .checked_mul(2)
                .filter(|n| *n > expected_count)
                .unwrap_or(1)
        } else {
            generation.unwrap().capacity()
        };
        if capacity <= expected_count {
            return Err(capacity_denial());
        }
        let buffer_bytes = u64::try_from(capacity)
            .ok()
            .and_then(|n| {
                n.checked_mul(std::mem::size_of::<
                    std::sync::Arc<std::sync::OnceLock<super::super::RecordedOutput>>,
                >() as u64)
            })
            .ok_or_else(capacity_denial)?;
        let path_bytes = tree_insert_bytes::<
            SemanticSource,
            BTreeMap<ProductBranchIncarnation, BTreeMap<u64, RecordedGeneration>>,
        >(self.by_source.len())
        .and_then(|n| {
            n.checked_add(tree_insert_bytes::<
                ProductBranchIncarnation,
                BTreeMap<u64, RecordedGeneration>,
            >(occurrences.map_or(0, |rows| rows.len()))?)
        })
        .and_then(|n| {
            n.checked_add(tree_insert_bytes::<u64, RecordedGeneration>(
                history.map_or(0, |rows| rows.len()),
            )?)
        })
        .ok_or_else(capacity_denial)?;
        let bytes = path_bytes
            .checked_add(buffer_bytes)
            .ok_or_else(capacity_denial)?;
        let future_outer = tree_work::<SemanticSource>(
            self.by_source
                .len()
                .checked_add(usize::from(occurrences.is_none()))
                .ok_or_else(work_denial)?,
        )
        .ok_or_else(work_denial)?;
        let future_middle = tree_work::<ProductBranchIncarnation>(
            occurrences
                .map_or(0, |rows| rows.len())
                .checked_add(usize::from(history.is_none()))
                .ok_or_else(work_denial)?,
        )
        .ok_or_else(work_denial)?;
        let future_inner = tree_work::<u64>(
            history
                .map_or(0, |rows| rows.len())
                .checked_add(usize::from(inserted_generation))
                .ok_or_else(work_denial)?,
        )
        .ok_or_else(work_denial)?;
        let work = outer
            .checked_add(middle)
            .and_then(|n| n.checked_add(inner))
            .and_then(|n| {
                n.checked_add(
                    future_outer
                        .checked_add(future_middle)?
                        .checked_add(future_inner)?
                        .checked_mul(2)?,
                )
            })
            .ok_or_else(work_denial)?;
        admission
            .charge_ordered_operations(9, work)
            .map_err(|_| work_denial())?;
        admission
            .admit_read_scratch(bytes)
            .map_err(|_| capacity_denial())?;
        let payload_work = if grow {
            u64::try_from(expected_count).map_err(|_| work_denial())?
        } else {
            0
        };
        admission
            .charge_external_work(payload_work.checked_add(2).ok_or_else(work_denial)?)
            .map_err(|_| work_denial())?;
        retained.reserve_additional(bytes)?;
        let mut replacement = if grow {
            let mut rows = Vec::new();
            rows.try_reserve_exact(capacity)
                .map_err(|_| capacity_denial())?;
            if let Some(generation) = generation {
                rows.extend(generation.iter().cloned());
            }
            Some(rows)
        } else {
            None
        };
        if inserted_generation {
            let rows = replacement.take().unwrap();
            assert!(self
                .by_source
                .entry(source.clone())
                .or_default()
                .entry(coordinate.occurrence)
                .or_default()
                .insert(coordinate.generation, rows)
                .is_none());
        }
        Ok(PreparedGenerationAppend {
            inserted_generation,
            replacement,
        })
    }

    pub(in crate::domain_computation::primary_graph::output_lineage::input_cutoff) fn publish_stable_generation(
        &mut self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        record: std::sync::Arc<std::sync::OnceLock<super::super::RecordedOutput>>,
        prepared: PreparedGenerationAppend,
    ) -> Option<RecordedGeneration> {
        let rows = self
            .by_source
            .get_mut(source)
            .unwrap()
            .get_mut(&coordinate.occurrence)
            .unwrap()
            .get_mut(&coordinate.generation)
            .unwrap();
        let retired = prepared
            .replacement
            .map(|replacement| std::mem::replace(rows, replacement));
        assert!(rows.len() < rows.capacity());
        rows.push(record);
        retired
    }

    pub(in crate::domain_computation::primary_graph::output_lineage::input_cutoff) fn abandon_stable_generation(
        &mut self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        prepared: &PreparedGenerationAppend,
    ) -> Option<RecordedGeneration> {
        if !prepared.inserted_generation {
            return None;
        }
        let occurrences = self.by_source.get_mut(source).unwrap();
        let history = occurrences.get_mut(&coordinate.occurrence).unwrap();
        let rows = history.remove(&coordinate.generation).unwrap();
        assert!(rows.is_empty());
        if history.is_empty() {
            occurrences.remove(&coordinate.occurrence);
        }
        if occurrences.is_empty() {
            self.by_source.remove(source);
        }
        Some(rows)
    }
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    denial(Kind::WorkBudgetExceeded)
}
fn capacity_denial() -> WorthQueryOutputDemandDenial {
    denial(Kind::RetentionBudgetExceeded)
}
