//! Accepted owner checkpoints retained as runs, never a second budget.
use worth_execution::{ChargedBytes, MapKernelContext, MapKernelStop};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in super::super) struct KernelSteps(Vec<(u64, u64)>);

impl KernelSteps {
    pub(in super::super) fn accepted(&mut self, units: u64) {
        if let Some((previous, repetitions)) = self.0.last_mut() {
            if *previous == units {
                *repetitions += 1;
                return;
            }
        }
        self.0.push((units, 1));
    }

    pub(in super::super) fn accounts_for(&self, work: u64, span: u64) -> bool {
        work == span
            && self.0.iter().try_fold(0_u64, |sum, (units, count)| {
                sum.checked_add(units.checked_mul(*count)?)
            }) == Some(work)
    }

    pub(in super::super) fn replay(
        &self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), MapKernelStop> {
        for (units, count) in &self.0 {
            for _ in 0..*count {
                context.checkpoint(*units)?;
            }
        }
        Ok(())
    }
}

impl ChargedBytes for KernelSteps {
    fn additional_charged_bytes(&self) -> u64 {
        u64::try_from(self.0.len()).expect("allocated checkpoint count fits")
            * u64::try_from(size_of::<(u64, u64)>()).expect("checkpoint size fits")
    }
}
