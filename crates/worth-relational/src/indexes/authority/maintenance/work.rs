use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};

pub(super) struct MaintenanceAdmissionRefusal;

pub(super) struct MaintenanceWork<'a> {
    pub(super) budget: DerivedIndexMaintenanceBudget,
    pub(super) counts: DerivedIndexMaintenanceWork,
    preparation: Option<&'a mut dyn FnMut(u64, u64) -> Result<(), MaintenanceAdmissionRefusal>>,
}

impl<'a> MaintenanceWork<'a> {
    pub(super) fn has_preparation(&self) -> bool {
        self.preparation.is_some()
    }
    pub(super) fn new(budget: DerivedIndexMaintenanceBudget) -> Self {
        Self {
            budget,
            counts: Default::default(),
            preparation: None,
        }
    }

    pub(super) fn admitted(
        budget: DerivedIndexMaintenanceBudget,
        preparation: &'a mut dyn FnMut(u64, u64) -> Result<(), MaintenanceAdmissionRefusal>,
    ) -> Self {
        Self {
            preparation: Some(preparation),
            ..Self::new(budget)
        }
    }

    /// Framework preparation extends the caller's authority. Native counters
    /// remain the existing algorithm's receipt; they do not include these copies.
    pub(super) fn prepare(&mut self, work: u64, bytes: u64) -> Result<(), Denial> {
        if let Some(prepare) = self.preparation.as_mut() {
            prepare(work, bytes).map_err(|_| Denial::WorkBudgetExceeded)?;
        }
        Ok(())
    }

    pub(super) fn charge(&mut self, units: usize) -> Result<(), Denial> {
        if units > self.remaining() {
            return Err(Denial::WorkBudgetExceeded);
        }
        self.prepare(units as u64, 0)?;
        self.counts.work_units += units;
        Ok(())
    }

    pub(super) fn remaining(&self) -> usize {
        self.budget
            .maximum_work_units
            .saturating_sub(self.counts.work_units)
    }

    /// The carried parent already funded this read before action. Keep the
    /// independent native receipt and ceiling without charging it a second time.
    pub(super) fn settle_prepaid(&mut self, units: usize) -> Result<(), Denial> {
        if units > self.remaining() {
            return Err(Denial::WorkBudgetExceeded);
        }
        self.counts.work_units += units;
        Ok(())
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
