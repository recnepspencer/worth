//! Request work may refuse a search; terminal cleanup cannot.
use super::{work_denial, InvalidationEditAdmission, WorthQueryOutputDemandDenial};
use core::convert::Infallible;

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
