//! Price selected occurrence walks before a registry mutation starts.
use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use super::{infallible, occurrence_rows, work_denial, OccurrenceSearchCharge};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use core::convert::Infallible;

/// Exclusive custody spans payment and execution. These edits may remove
/// rows, but must neither insert rows nor grow an occurrence before walking.
/// The existing immutable whole-map receipt is a different proof and keeps
/// its immutable borrow. This receipt pays occurrence-local upper bounds.
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct PricedMutation<
    'a,
> {
    state: &'a mut DemandRegistryState,
    charge: PricedMutationWalk,
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct PricedMutationWalk
{
    paths: u64,
    rows: u64,
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn price_mutation<
    'a,
>(
    state: &'a mut DemandRegistryState,
    admission: &mut InvalidationEditAdmission,
    walks: &[(WorthQueryOutputDemandKey, u64)],
) -> Result<PricedMutation<'a>, WorthQueryOutputDemandDenial> {
    let mut paths = 0u64;
    let mut rows = 0u64;
    let mut navigation = 0u64;
    for (key, count) in walks {
        let mut occurrence_count = 0u64;
        // The quote itself is a paid walk, including its row inspections.
        let navigation_before = admission.charged_navigation();
        let selected = occurrence_rows(&state.records, key, admission)?;
        let path_navigation = admission.charged_navigation() - navigation_before;
        for _ in selected {
            admission.inspect_row()?;
            occurrence_count = occurrence_count.checked_add(1).ok_or_else(work_denial)?;
        }
        let inspected = occurrence_count
            .checked_mul(*count)
            .ok_or_else(work_denial)?;
        navigation = path_navigation
            .checked_mul(*count)
            .and_then(|price| navigation.checked_add(price))
            .ok_or_else(work_denial)?;
        paths = paths.checked_add(*count).ok_or_else(work_denial)?;
        rows = rows.checked_add(inspected).ok_or_else(work_denial)?;
    }
    let work = paths
        .checked_mul(2)
        .and_then(|price| price.checked_add(rows))
        .ok_or_else(work_denial)?;
    admission
        .charge_ordered_operations(work, navigation)
        .map_err(|_| work_denial())?;
    Ok(PricedMutation {
        state,
        charge: PricedMutationWalk { paths, rows },
    })
}

impl PricedMutation<'_> {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn complete<
        T,
    >(
        mut self,
        operation: impl FnOnce(
            &mut DemandRegistryState,
            &mut PricedMutationWalk,
        ) -> Result<T, Infallible>,
    ) -> T {
        infallible(operation(self.state, &mut self.charge))
    }
}

impl OccurrenceSearchCharge for PricedMutationWalk {
    type Refusal = Infallible;
    fn occurrence_path(&mut self, _: usize) -> Result<(), Infallible> {
        self.paths = self
            .paths
            .checked_sub(1)
            .expect("selected mutation walks were priced");
        Ok(())
    }
    fn inspect_row(&mut self) -> Result<(), Infallible> {
        self.rows = self
            .rows
            .checked_sub(1)
            .expect("selected occurrence rows were priced");
        Ok(())
    }
}
