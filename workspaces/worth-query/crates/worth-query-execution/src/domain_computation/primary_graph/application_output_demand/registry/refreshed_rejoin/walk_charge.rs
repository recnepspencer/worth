//! Request work may refuse a search; terminal cleanup cannot.
use super::super::record_map::DemandRecords;
use super::{
    newest_matching_row, refresh_order, search_stale_owner, work_denial, InvalidationEditAdmission,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandKey,
};
use core::convert::Infallible;
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) enum WholeMapPasses
{
    One,
    Two,
}

/// A whole-map payment retains an immutable borrow of precisely those records.
/// Its fields are sealed here; registry callers can only obtain it by paying.
///
/// ```compile_fail,E0603
/// use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::walk_charge::PrepaidOccurrenceWalk;
/// let _receipt = PrepaidOccurrenceWalk { records: &Default::default() };
/// ```
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct PrepaidOccurrenceWalk<
    'records,
> {
    records: &'records DemandRecords,
}

/// Keep combined payments atomic, including the two-pass reclaim and the
/// joined-Ready walk's four additional units. No successful charge, no receipt.
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn charge_whole_map<
    'records,
>(
    records: &'records DemandRecords,
    admission: &mut InvalidationEditAdmission,
    passes: WholeMapPasses,
    additional_work: u64,
) -> Result<PrepaidOccurrenceWalk<'records>, CompanionPreflightStop> {
    let rows =
        u64::try_from(records.len()).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
    let rows = match passes {
        WholeMapPasses::One => rows,
        WholeMapPasses::Two => rows.saturating_mul(2),
    };
    let work = rows
        .checked_add(additional_work)
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    admission.charge_external_work(work)?;
    Ok(PrepaidOccurrenceWalk { records })
}

impl PrepaidOccurrenceWalk<'_> {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn stale_owner(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> bool {
        match search_stale_owner(self.records, key, self) {
            Ok(awaited) => awaited,
            Err(never) => match never {},
        }
    }

    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn newest(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<WorthQueryOutputDemandKey> {
        match newest_matching_row(self.records, key, self, |candidate, _| {
            refresh_order(candidate, key) == Some(std::cmp::Ordering::Greater)
        }) {
            Ok(newest) => newest,
            Err(never) => match never {},
        }
    }
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) trait OccurrenceSearchCharge
{
    type Refusal;

    fn occurrence_path(&mut self, records: usize) -> Result<(), Self::Refusal>;
    fn inspect_row(&mut self) -> Result<(), Self::Refusal>;
}

impl OccurrenceSearchCharge for InvalidationEditAdmission {
    type Refusal = WorthQueryOutputDemandDenial;

    fn occurrence_path(&mut self, records: usize) -> Result<(), Self::Refusal> {
        let path = u64::from(usize::BITS - records.leading_zeros())
            .checked_mul(11)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(work_denial)?;
        self.charge_ordered_operations(2, path.checked_mul(2).ok_or_else(work_denial)?)
            .map_err(|_| work_denial())
    }

    fn inspect_row(&mut self) -> Result<(), Self::Refusal> {
        self.charge_external_work(1).map_err(|_| work_denial())
    }
}

impl OccurrenceSearchCharge for () {
    type Refusal = Infallible;

    fn occurrence_path(&mut self, _: usize) -> Result<(), Self::Refusal> {
        Ok(())
    }

    fn inspect_row(&mut self) -> Result<(), Self::Refusal> {
        Ok(())
    }
}

impl OccurrenceSearchCharge for PrepaidOccurrenceWalk<'_> {
    type Refusal = Infallible;

    fn occurrence_path(&mut self, _: usize) -> Result<(), Self::Refusal> {
        Ok(())
    }

    fn inspect_row(&mut self) -> Result<(), Self::Refusal> {
        Ok(())
    }
}
