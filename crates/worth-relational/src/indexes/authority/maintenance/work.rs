use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};

pub(super) struct MaintenanceAdmissionRefusal;

enum MaintenancePayer<'a, 'run, 'lease> {
    Request(&'a mut worth_execution::MapKernelContext<'run, 'lease>),
    Admission(&'a mut dyn FnMut(u64, u64) -> Result<(), MaintenanceAdmissionRefusal>),
    #[cfg(test)]
    Test,
}

pub(super) struct MaintenanceWork<'a, 'run, 'lease> {
    pub(super) budget: DerivedIndexMaintenanceBudget,
    pub(super) counts: DerivedIndexMaintenanceWork,
    payer: MaintenancePayer<'a, 'run, 'lease>,
}

impl<'a, 'run, 'lease> MaintenanceWork<'a, 'run, 'lease> {
    pub(super) fn has_preparation(&self) -> bool {
        matches!(self.payer, MaintenancePayer::Admission(_))
    }
    #[cfg(test)]
    pub(super) fn new(budget: DerivedIndexMaintenanceBudget) -> Self {
        Self {
            budget,
            counts: Default::default(),
            payer: MaintenancePayer::Test,
        }
    }

    pub(super) fn admitted(
        budget: DerivedIndexMaintenanceBudget,
        preparation: &'a mut dyn FnMut(u64, u64) -> Result<(), MaintenanceAdmissionRefusal>,
    ) -> Self {
        Self {
            budget,
            counts: Default::default(),
            payer: MaintenancePayer::Admission(preparation),
        }
    }

    pub(super) fn request_paid(
        budget: DerivedIndexMaintenanceBudget,
        context: &'a mut worth_execution::MapKernelContext<'run, 'lease>,
    ) -> Self {
        Self {
            budget,
            counts: Default::default(),
            payer: MaintenancePayer::Request(context),
        }
    }

    /// Framework preparation extends the caller's authority. Native counters
    /// remain the existing algorithm's receipt; they do not include these copies.
    pub(super) fn prepare(&mut self, work: u64, bytes: u64) -> Result<(), Denial> {
        match &mut self.payer {
            MaintenancePayer::Admission(prepare) => {
                prepare(work, bytes).map_err(|_| Denial::WorkBudgetExceeded)
            }
            // Native work is paid by charge; framework preparation consults the
            // same active request without duplicating the native receipt.
            MaintenancePayer::Request(context) => checkpoint(context, 0),
            #[cfg(test)]
            MaintenancePayer::Test => Ok(()),
        }
    }

    pub(super) fn charge(&mut self, units: usize) -> Result<(), Denial> {
        #[cfg(test)]
        super::charge_rendezvous::meet();
        if let MaintenancePayer::Request(context) = &mut self.payer {
            checkpoint(context, 0)?;
            if units as u64 > context.remaining_work() {
                checkpoint(context, units as u64)?;
            }
        }
        if units > self.native_remaining() {
            return Err(Denial::WorkBudgetExceeded);
        }
        match &mut self.payer {
            MaintenancePayer::Request(context) => checkpoint(context, units as u64)?,
            MaintenancePayer::Admission(prepare) => {
                prepare(units as u64, 0).map_err(|_| Denial::WorkBudgetExceeded)?
            }
            #[cfg(test)]
            MaintenancePayer::Test => {}
        }
        self.counts.work_units += units;
        Ok(())
    }

    pub(super) fn remaining(&self) -> usize {
        let native = self.native_remaining();
        match &self.payer {
            MaintenancePayer::Request(context) => usize::try_from(context.remaining_work())
                .map_or(native, |remaining| native.min(remaining)),
            _ => native,
        }
    }

    fn native_remaining(&self) -> usize {
        self.budget
            .maximum_work_units
            .saturating_sub(self.counts.work_units)
    }

    pub(super) fn bounded_scan_denial(&mut self) -> Denial {
        let native = self.native_remaining();
        if let MaintenancePayer::Request(context) = &mut self.payer {
            let remaining = context.remaining_work();
            if remaining <= native as u64 {
                let Some(required) = remaining.checked_add(1) else {
                    return Denial::Execution(
                        crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                    );
                };
                return checkpoint(context, required)
                    .expect_err("bounded scan requires more than the carried request remainder");
            }
        }
        Denial::WorkBudgetExceeded
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

fn checkpoint(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    units: u64,
) -> Result<(), Denial> {
    context.checkpoint(units).map_err(|stop| {
        Denial::Execution(crate::execution::kernel_failure(
            worth_execution::MapKernelFailure::Stop(stop),
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_scan_quote_is_bounded_by_the_callers_remaining_work() {
        let host = crate::test_execution_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let budget = DerivedIndexMaintenanceBudget {
            maximum_work_units: 100,
            maximum_cold_record_slots: 0,
            maximum_derived_rows: 0,
        };
        let (result, _) = execution
            .run(worth_execution::ExecutionWorkCeiling::new(2), |_| {
                super::super::request_payment::run(&execution, budget, |work| Ok(work.remaining()))
            })
            .unwrap();
        assert_eq!(result.unwrap().0, 2);
    }

    #[test]
    fn bounded_child_scan_stop_retains_the_budget_owner() {
        for (caller_limit, native_limit, expected) in [
            (
                2,
                100,
                Denial::Execution(crate::execution::RelationalExecutionDenialCause::WorkExhausted),
            ),
            (100, 2, Denial::WorkBudgetExceeded),
        ] {
            let host = crate::test_execution_policy::serial_request();
            let execution = worth_execution::ExecutionRequest::serial(&host);
            let budget = DerivedIndexMaintenanceBudget {
                maximum_work_units: native_limit,
                maximum_cold_record_slots: 0,
                maximum_derived_rows: 0,
            };
            let (result, report) = execution
                .run(
                    worth_execution::ExecutionWorkCeiling::new(caller_limit),
                    |_| {
                        super::super::request_payment::run(&execution, budget, |work| {
                            work.charge(2)?;
                            Err::<(), _>(work.bounded_scan_denial())
                        })
                    },
                )
                .unwrap();
            let denied = result.unwrap_err();
            assert_eq!(denied.kind, expected);
            assert_eq!(denied.work.work_units, 2);
            assert_eq!(report.charged_work(), 2);
        }
    }
}
