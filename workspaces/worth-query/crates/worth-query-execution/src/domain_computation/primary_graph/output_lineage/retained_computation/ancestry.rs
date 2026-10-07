//! One prior-selection rule: local record, then exact captured fork horizons.
use super::super::{
    invalidation::InvalidationEditAdmission, prepared_slot::tree_work, ProductCoordinate,
    RecordedOutput, SemanticSource, WorthQueryApplicationOutputLineage,
};
use std::sync::{Arc, OnceLock};
use worth_relational::facade::mvcc::CompanionPreflightStop;
impl WorthQueryApplicationOutputLineage {
    /// Prepay the later pin scan before contacting publication. If forks grow
    /// meanwhile, retention preserves prior custody without performing a scan.
    pub(in crate::domain_computation::primary_graph::output_lineage) fn prepay_computation_fork_scan(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<usize, super::super::super::WorthQueryOutputDemandDenial> {
        let forks = self.origins.len();
        let work = u64::try_from(forks)
            .ok()
            .and_then(|count| count.checked_mul(2))
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| {
                super::super::prepared_slot::denial(
                    super::super::super::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                )
            })?;
        admission.charge_external_work(work).map_err(|_| {
            super::super::prepared_slot::denial(
                super::super::super::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            )
        })?;
        Ok(forks)
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph::output_lineage) fn prepay_computation_fork_scan_for_test(
        &self,
    ) -> usize {
        self.prepay_computation_fork_scan(&mut InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: 4096,
                maximum_preparation_bytes: 4096,
            },
        ))
        .unwrap()
    }

    pub(in crate::domain_computation::primary_graph::output_lineage) fn computation_cell_in_ancestry(
        &self,
        source: &SemanticSource,
        mut coordinate: ProductCoordinate,
        partition: [u8; 32],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Arc<OnceLock<RecordedOutput>>>, CompanionPreflightStop> {
        loop {
            if let Some((generation, slot)) = self
                .partition_index
                .latest_admitted(source, coordinate, partition, admission)?
            {
                return self
                    .recorded_cell_at_partition_slot_admitted(
                        source,
                        coordinate.occurrence,
                        generation,
                        partition,
                        slot,
                        admission,
                    )
                    .map(Some);
            }
            admission.charge_external_work(
                tree_work::<worth_runtime_world::facade::ProductBranchIncarnation>(
                    self.origins.len(),
                )
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
            )?;
            let Some(origin) = self.origins.get(&coordinate.occurrence) else {
                return Ok(None);
            };
            coordinate = *origin;
        }
    }
    /// Called only for the exact latest local record displaced by publication.
    /// No subsequent local record existed when a retained fork was captured.
    pub(in crate::domain_computation::primary_graph::output_lineage) fn fork_pins_computation(
        &self,
        recorded: &RecordedOutput,
    ) -> bool {
        let coordinate = recorded.settlement_identity.coordinate();
        self.origins.values().any(|origin| {
            origin.occurrence == coordinate.occurrence && origin.generation >= coordinate.generation
        })
    }
}
