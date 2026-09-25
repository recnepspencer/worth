use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};

pub(super) struct MaintenanceWork {
    pub(super) budget: DerivedIndexMaintenanceBudget,
    pub(super) counts: DerivedIndexMaintenanceWork,
}

impl MaintenanceWork {
    pub(super) fn new(budget: DerivedIndexMaintenanceBudget) -> Self {
        Self {
            budget,
            counts: Default::default(),
        }
    }

    pub(super) fn charge(&mut self, units: usize) -> Result<(), Denial> {
        if units > self.remaining() {
            return Err(Denial::WorkBudgetExceeded);
        }
        self.counts.work_units += units;
        Ok(())
    }

    pub(super) fn remaining(&self) -> usize {
        self.budget
            .maximum_work_units
            .saturating_sub(self.counts.work_units)
    }

    pub(super) fn read(&mut self) -> Result<(), Denial> {
        self.charge(1)?;
        self.counts.record_reads += 1;
        Ok(())
    }

    pub(super) fn derive_row(&mut self) -> Result<(), Denial> {
        if self.counts.derived_rows >= self.budget.maximum_derived_rows {
            return Err(Denial::WorkBudgetExceeded);
        }
        self.charge(1)?;
        self.counts.derived_rows += 1;
        Ok(())
    }

    pub(super) fn deny(&self, kind: Denial) -> DerivedIndexMaintenanceDenial {
        DerivedIndexMaintenanceDenial {
            kind,
            work: self.counts,
        }
    }
}
