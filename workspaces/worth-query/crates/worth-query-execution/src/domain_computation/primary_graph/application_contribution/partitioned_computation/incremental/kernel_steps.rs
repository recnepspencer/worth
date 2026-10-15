//! Accepted owner checkpoints retained as runs, never a second budget.
use worth_execution::{ChargedBytes, MapKernelContext, MapKernelStop};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in super::super) struct KernelSteps(Vec<(u64, u64)>);

impl KernelSteps {
    pub(in super::super) fn accepted(
        &mut self,
        units: u64,
        mut reserve: impl FnMut(u64) -> bool,
    ) -> bool {
        if let Some((previous, repetitions)) = self.0.last_mut()
            && *previous == units
        {
            let Some(next) = repetitions.checked_add(1) else {
                return false;
            };
            *repetitions = next;
            return true;
        }
        if self.0.len() == self.0.capacity() {
            let capacity = self.0.capacity().max(1);
            let bytes = u64::try_from(capacity).ok().and_then(|n| {
                n.checked_mul(
                    u64::try_from(size_of::<(u64, u64)>()).expect("checkpoint layout fits u64"),
                )
            });
            if !bytes.is_some_and(&mut reserve) || self.0.try_reserve_exact(capacity).is_err() {
                return false;
            }
        }
        self.0.push((units, 1));
        true
    }

    pub(in super::super) fn units(&self) -> u64 {
        self.0
            .iter()
            .try_fold(0_u64, |sum, (units, count)| {
                sum.checked_add(units.checked_mul(*count)?)
            })
            .expect("accepted checkpoints are bounded by their execution meter")
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
        u64::try_from(self.0.capacity()).expect("allocated checkpoint capacity fits")
            * u64::try_from(size_of::<(u64, u64)>()).expect("checkpoint size fits")
    }
}
