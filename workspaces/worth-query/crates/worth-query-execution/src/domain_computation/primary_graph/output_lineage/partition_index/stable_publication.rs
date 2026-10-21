//! Prepared replacement of one derived partition locator under lineage custody.

use super::OutputPartitionIndex;

use crate::domain_computation::primary_graph::output_lineage::prepared_slot::denial;
use crate::domain_computation::primary_graph::output_lineage::ProductCoordinate;
use crate::domain_computation::primary_graph::output_lineage::SemanticSource;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind as Kind;
use std::sync::Arc;
use std::sync::OnceLock;

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
    ) -> Result<PreparedStablePartitionLocator, WorthQueryOutputDemandDenial> {
        let occurrences = self.slots.get(source);
        let partitions = occurrences.and_then(|rows| rows.get(&coordinate.occurrence));
        let generations = partitions.and_then(|rows| rows.get(&Some(partition)));
        let existing = generations.and_then(|rows| rows.get(&coordinate.generation));
        if existing.is_some_and(|cell| cell.get().is_none()) {
            return Err(denial(Kind::SchedulingDeferred));
        }
        let inserted_vacancy = existing.is_none();
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
