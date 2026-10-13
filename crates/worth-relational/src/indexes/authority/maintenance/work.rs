use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};

#[path = "comparison_payment.rs"]
mod comparison_payment;

pub(super) struct MaintenanceAdmissionRefusal;

enum MaintenancePayer<'a, 'run, 'lease> {
    Request(
        &'a mut worth_execution::MapKernelContext<'run, 'lease>,
        &'a mut worth_execution::ExecutionMemoryReservation,
    ),
    Admission(&'a mut dyn FnMut(u64, u64) -> Result<(), MaintenanceAdmissionRefusal>),
    #[cfg(test)]
    Test,
}

pub(super) struct MaintenanceWork<'a, 'run, 'lease> {
    pub(super) budget: DerivedIndexMaintenanceBudget,
    counts: DerivedIndexMaintenanceWork,
    payer: MaintenancePayer<'a, 'run, 'lease>,
}

impl<'a, 'run, 'lease> MaintenanceWork<'a, 'run, 'lease> {
    pub(super) fn counts(&self) -> DerivedIndexMaintenanceWork {
        self.counts
    }
    pub(super) fn search_payment(
        &mut self,
        rows: usize,
    ) -> Result<comparison_payment::ComparisonPayment<'_, 'a, 'run, 'lease>, Denial> {
        comparison_payment::ComparisonPayment::for_search(self, rows)
    }
    pub(super) fn note_patch_records(&mut self, count: usize) {
        self.counts.patch_records += count;
    }
    pub(super) fn note_cold_record_slots(&mut self, count: usize) {
        self.counts.cold_record_slots += count;
    }
    pub(super) fn note_seek_comparisons(&mut self, count: usize) {
        self.counts.seek_comparisons += count;
    }
    pub(super) fn note_entry_edits(&mut self, count: usize) {
        self.counts.entry_edits += count;
    }
    pub(super) fn note_path_copy_units_reserved(&mut self, count: usize) {
        self.counts.path_copy_units_reserved += count;
    }
    pub(super) fn note_adjacency_work_units(&mut self, count: usize) {
        self.counts.adjacency_work_units += count;
    }
    pub(super) fn note_reused_generations(&mut self, count: usize) {
        self.counts.reused_generations += count;
    }
    pub(super) fn note_seeded_generations(&mut self, count: usize) {
        self.counts.seeded_generations += count;
    }
    pub(super) fn reserve_generation_publications(&mut self, count: usize) {
        self.counts.generation_publications_reserved = count;
    }
    pub(super) fn has_preparation(&self) -> bool {
        matches!(self.payer, MaintenancePayer::Admission(_))
    }
    pub(super) fn prepares_allocations(&self) -> bool {
        match self.payer {
            MaintenancePayer::Request(..) | MaintenancePayer::Admission(_) => true,
            #[cfg(test)]
            MaintenancePayer::Test => false,
        }
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
        memory: &'a mut worth_execution::ExecutionMemoryReservation,
    ) -> Self {
        Self {
            budget,
            counts: Default::default(),
            payer: MaintenancePayer::Request(context, memory),
        }
    }

    /// Framework preparation extends the caller's authority. Native counters
    /// remain the existing algorithm's receipt; they do not include these copies.
    /// The admitted caller pays preparation work; request callers pay native work in `charge`.
    pub(super) fn prepare(&mut self, work: u64, bytes: u64) -> Result<(), Denial> {
        match &mut self.payer {
            MaintenancePayer::Admission(prepare) => {
                prepare(work, bytes).map_err(|_| Denial::WorkBudgetExceeded)
            }
            // Native work is paid by charge; framework preparation consults the
            // same active request without duplicating the native receipt.
            MaintenancePayer::Request(context, memory) => {
                checkpoint(context, 0)?;
                let total = memory
                    .bytes()
                    .checked_add(bytes)
                    .ok_or(Denial::WorkBudgetExceeded)?;
                memory.resize(total).map_err(|denial| {
                    Denial::Execution(crate::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(denial),
                    ))
                })
            }
            #[cfg(test)]
            MaintenancePayer::Test => Ok(()),
        }
    }

    pub(super) fn charge(&mut self, units: usize) -> Result<(), Denial> {
        #[cfg(test)]
        super::charge_rendezvous::meet();
        if let MaintenancePayer::Request(context, _) = &mut self.payer {
            checkpoint(context, 0)?;
            if units as u64 > context.remaining_work() {
                checkpoint(context, units as u64)?;
            }
        }
        if units > self.native_remaining() {
            return Err(Denial::WorkBudgetExceeded);
        }
        match &mut self.payer {
            MaintenancePayer::Request(context, _) => checkpoint(context, units as u64)?,
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
            MaintenancePayer::Request(context, _) => usize::try_from(context.remaining_work())
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
        if let MaintenancePayer::Request(context, _) = &mut self.payer {
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
        assert_eq!(result.unwrap().0.value, 2);
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
