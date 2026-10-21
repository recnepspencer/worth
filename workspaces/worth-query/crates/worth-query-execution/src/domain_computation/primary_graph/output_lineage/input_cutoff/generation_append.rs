//! Selected generation storage prepared before stable source publication.

use super::super::prepared_slot::denial;
use super::super::ProductCoordinate;
use super::super::RecordedGeneration;
use super::super::SemanticSource;
use super::super::WorthQueryApplicationOutputLineage;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind as Kind;

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
    ) -> Result<PreparedGenerationAppend, WorthQueryOutputDemandDenial> {
        let occurrences = self.by_source.get(source);
        let history = occurrences.and_then(|rows| rows.get(&coordinate.occurrence));
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

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    denial(Kind::RetentionBudgetExceeded)
}
