//! A search payment borrows its account; only its funded parent can mint it.
use super::MaintenanceWork;
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;

pub(in crate::indexes::authority::maintenance) struct ComparisonPayment<'work, 'payer, 'run, 'lease>
{
    work: &'work mut MaintenanceWork<'payer, 'run, 'lease>,
    prepaid: Option<usize>,
}

impl<'work, 'payer, 'run, 'lease> ComparisonPayment<'work, 'payer, 'run, 'lease> {
    pub(super) fn for_search(
        work: &'work mut MaintenanceWork<'payer, 'run, 'lease>,
        rows: usize,
    ) -> Result<Self, Denial> {
        // This is the same parent lookup payment as the admitted lane used
        // before receipts. Its conservative bound funds every comparison.
        let units = super::super::preparation::lookup_work(rows, 1)?;
        work.prepare(units as u64, 0)?;
        let prepaid = work.search_is_prepared().then_some(units);
        Ok(Self { work, prepaid })
    }

    pub(in crate::indexes::authority::maintenance) fn comparison(&mut self) -> Result<(), Denial> {
        match &mut self.prepaid {
            Some(credit) => {
                if *credit == 0 || self.work.native_remaining() == Some(0) {
                    return Err(Denial::WorkBudgetExceeded);
                }
                let next = self
                    .work
                    .counts
                    .work_units
                    .checked_add(1)
                    .ok_or(Denial::Execution(
                        crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                    ))?;
                *credit -= 1;
                self.work.counts.work_units = next;
                Ok(())
            }
            None => self.work.charge(1),
        }
    }
}
