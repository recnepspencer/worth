use super::{KernelContext, KernelMeter, KernelStop, NestedStopOwner};

impl KernelContext<'_, '_> {
    /// Account work completed by a domain-local checked meter after it stops.
    /// This does not authorize new work or clear the original stop; it keeps
    /// cancellation/deadline reports faithful to work already performed.
    pub fn account_completed_work(&mut self, units: u64) -> Result<(), KernelStop> {
        if units > 0 {
            crate::authority::consult_observation::charged_prefix(|| {
                super::ACTIVE_METER.with(|active| {
                    active
                        .borrow()
                        .iter()
                        .try_fold(units, |sum, meter| sum.checked_add(meter.borrow().work))
                        .unwrap_or(u64::MAX)
                })
            });
        }
        let mut meter = self.meter.borrow_mut();
        let result = (|| {
            let next = meter
                .work
                .checked_add(units)
                .ok_or(KernelStop::WorkCounterOverflow)?;
            let span = meter
                .span
                .checked_add(units)
                .ok_or(KernelStop::WorkCounterOverflow)?;
            if meter.limits.ceiling.is_some_and(|ceiling| next > ceiling) {
                return Err(KernelStop::WorkCeiling);
            }
            meter.work = next;
            meter.span = span;
            Ok(())
        })();
        if let Err(stop) = result {
            meter.checkpoint_stop.get_or_insert(stop);
            if stop == KernelStop::WorkCounterOverflow
                || meter.limits.ceiling_owner == NestedStopOwner::Request
            {
                meter.nested_stop.record(NestedStopOwner::Request);
            }
        }
        result
    }

    /// Work units charged to this kernel's existing meter.
    pub fn charged_work(&self) -> u64 {
        self.meter.borrow().work
    }

    pub fn remaining_work(&self) -> Option<u64> {
        let meter = self.meter.borrow();
        meter
            .limits
            .ceiling
            .map(|ceiling| ceiling.saturating_sub(meter.work))
    }

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
        crate::authority::consult_observation::reached();
        if units > 0 {
            crate::authority::consult_observation::charged_prefix(|| {
                super::ACTIVE_METER.with(|active| {
                    active
                        .borrow()
                        .iter()
                        .try_fold(units, |sum, meter| sum.checked_add(meter.borrow().work))
                        .unwrap_or(u64::MAX)
                })
            });
        }
        let mut meter = self.meter.borrow_mut();
        if let Some(stop) = meter.checkpoint_stop {
            crate::authority::consult_observation::stopped(Some((
                std::rc::Rc::as_ptr(&self.meter) as usize,
                true,
            )));
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
            if meter.limits.ceiling.is_some_and(|ceiling| next > ceiling) {
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
            crate::authority::consult_observation::stopped(Some((
                std::rc::Rc::as_ptr(&self.meter) as usize,
                false,
            )));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::run_scope;
    use crate::tests::{authority, request, TEST_LOCK};
    #[test]
    fn refused_completed_work_keeps_the_checkpoint_stop() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let lease = authority().request_lease(request(1, 2_000, 3)).unwrap();
        let outcome = run_scope(Some(&lease), 0, 0, |context| {
            assert_eq!(context.account_completed_work(2), Ok(()));
            assert_eq!(
                context.account_completed_work(2),
                Err(KernelStop::WorkCeiling)
            );
            assert_eq!(context.checkpoint_stop(), Some(KernelStop::WorkCeiling));
            assert_eq!(context.checkpoint(0), Err(KernelStop::WorkCeiling));
            assert_eq!(context.cost().0, 2);
            Err::<(), _>(crate::backend::KernelFailure::<()>::Stop(
                KernelStop::WorkCeiling,
            ))
        });
        assert!(matches!(
            outcome.result,
            Err(crate::backend::ScopeStop::Failure(
                crate::backend::KernelFailure::Stop(KernelStop::WorkCeiling)
            ))
        ));
        assert_eq!(outcome.report.charged_work(), 2);
    }
}
