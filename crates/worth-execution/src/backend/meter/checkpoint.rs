use super::{KernelContext, KernelMeter, KernelStop, NestedStopOwner};

impl KernelContext<'_, '_> {
    /// Kernel checkpoint enforces the local remaining ceiling before work is
    /// done, while canonical settlement determines the accepted prefix.
    pub fn checkpoint(&mut self, units: u64) -> Result<(), KernelStop> {
        super::super::checkpoint_observation::record(units);
        self.checkpoint_inner(units, false)
    }

    /// Select the canonical frontier prefix after all speculative tasks joined.
    /// Ownership is already latched; cancellation, ceilings and counters still
    /// govern this selection. Ordinary kernels cannot enter this lane.
    pub(crate) fn checkpoint_frontier(&mut self, units: u64) -> Result<(), KernelStop> {
        self.checkpoint_inner(units, true)
    }

    fn checkpoint_inner(&mut self, units: u64, settling_frontier: bool) -> Result<(), KernelStop> {
        let mut meter = self.meter.borrow_mut();
        if let Some(stop) = meter.checkpoint_stop {
            return Err(stop);
        }
        let result = (|| {
            let KernelMeter {
                limits,
                nested_stop,
                ..
            } = &mut *meter;
            limits.safe_point(nested_stop)?;
            let owner = meter.nested_stop.owner();
            if owner.is_some() && !(settling_frontier && owner == Some(NestedStopOwner::Request)) {
                return Err(KernelStop::NestedStopped);
            }
            let next = meter
                .work
                .checked_add(units)
                .ok_or(KernelStop::WorkCounterOverflow)?;
            if next > meter.limits.ceiling {
                return Err(KernelStop::WorkCeiling);
            }
            let span = meter
                .span
                .checked_add(units)
                .ok_or(KernelStop::WorkCounterOverflow)?;
            meter.work = next;
            meter.span = span;
            Ok(())
        })();
        if let Err(stop) = result {
            if matches!(stop, KernelStop::WorkCounterOverflow)
                || (stop == KernelStop::WorkCeiling
                    && meter.limits.ceiling_owner == NestedStopOwner::Request)
            {
                meter.nested_stop.record(NestedStopOwner::Request);
            }
            // The computational latch is the only memory of an untaken cause.
            // A checkpoint cannot turn it into a second, irreversible latch.
            if stop != KernelStop::NestedStopped
                || meter.nested_stop.owner() == Some(NestedStopOwner::Request)
            {
                meter.checkpoint_stop = Some(stop);
            }
        }
        result
    }
}
