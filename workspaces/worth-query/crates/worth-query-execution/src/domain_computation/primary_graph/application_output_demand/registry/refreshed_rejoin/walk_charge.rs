//! Request work may refuse a search; terminal cleanup cannot.
use super::super::record_map::DemandRecords;
use super::{
    awaited_by_stale_owner, newest_matching_row, refresh_order, work_denial,
    InvalidationEditAdmission, WorthQueryOutputDemandDenial, WorthQueryOutputDemandKey,
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
/// Each search consumes a paid pass; the consuming chain call serves disjoint
/// occurrences in one pass. No caller receives the private pass payer.
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct PrepaidOccurrenceWalk<
    'records,
> {
    records: &'records DemandRecords,
    remaining: u8,
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
    let remaining = match passes {
        WholeMapPasses::One => 1,
        WholeMapPasses::Two => 2,
    };
    let rows = rows.saturating_mul(u64::from(remaining));
    let work = rows
        .checked_add(additional_work)
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    admission.charge_external_work(work)?;
    Ok(PrepaidOccurrenceWalk { records, remaining })
}

impl PrepaidOccurrenceWalk<'_> {
    /// Consume one paid pass for a read-only mutation plan on this map.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn visit_rows(
        mut self,
        mut visit: impl FnMut(&WorthQueryOutputDemandKey, &super::super::DemandRecord),
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let pass = self.take_pass()?;
        for (key, row) in pass.records {
            visit(key, row);
        }
        Ok(())
    }

    fn take_pass(&mut self) -> Result<PaidPass<'_>, WorthQueryOutputDemandDenial> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| super::WalkKind::PrepaidPass.denial())?;
        Ok(PaidPass {
            records: self.records,
        })
    }

    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn stale_owner(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let mut pass = self.take_pass()?;
        Ok(super::infallible(awaited_by_stale_owner(
            pass.records,
            key,
            &mut pass,
        )))
    }

    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn newest(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
        let mut pass = self.take_pass()?;
        Ok(super::infallible(newest_matching_row(
            pass.records,
            key,
            &mut pass,
            |candidate, _| refresh_order(candidate, key) == Some(std::cmp::Ordering::Greater),
        )))
    }

    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn replaced_ready(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
        let mut pass = self.take_pass()?;
        Ok(super::infallible(
            super::super::succession::replaced_ready_key(pass.records, key, &mut pass),
        ))
    }

    /// One consuming pass over disjoint occurrences, including their upstream
    /// occurrences. A discovered occurrence enters the chain only once.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn chain(
        mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<Vec<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
        let mut pass = self.take_pass()?;
        let mut chain = vec![key.clone()];
        let mut next = 0;
        while let Some(occurrence) = chain.get(next).cloned() {
            next += 1;
            let rows =
                super::infallible(super::occurrence_rows(pass.records, &occurrence, &mut pass));
            for (_, record) in rows {
                for claimed in &record.prerequisites {
                    if !chain.iter().any(|known| known.same_occurrence(claimed)) {
                        chain.push(claimed.as_ref().clone());
                    }
                }
            }
        }
        Ok(chain)
    }
}

// Only receipt methods may construct or use a paid pass. The records cannot
// be supplied separately by a registry caller, or changed during the pass.
struct PaidPass<'records> {
    records: &'records DemandRecords,
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

impl OccurrenceSearchCharge for super::super::lifecycle::release::CleanupWalk {
    type Refusal = Infallible;

    fn occurrence_path(&mut self, _: usize) -> Result<(), Self::Refusal> {
        Ok(())
    }

    fn inspect_row(&mut self) -> Result<(), Self::Refusal> {
        Ok(())
    }
}

impl OccurrenceSearchCharge for PaidPass<'_> {
    type Refusal = Infallible;

    fn occurrence_path(&mut self, _: usize) -> Result<(), Self::Refusal> {
        Ok(())
    }

    fn inspect_row(&mut self) -> Result<(), Self::Refusal> {
        Ok(())
    }
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn infallible<
    T,
>(
    result: Result<T, Infallible>,
) -> T {
    match result {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
