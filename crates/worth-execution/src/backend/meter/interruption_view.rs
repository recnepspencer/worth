//! Capture controls from the actual invoking meter without entering another run.
use super::{run_invoking_preparation, KernelContext, ACTIVE_METER};
use crate::authority::{
    interruption_backing_lease, unrelated_interruption_context, InterruptionViewBuilder,
};
use crate::{ExecutionInterruptionView, ExecutionInterruptionViewDenial, ExecutionResourceLease};
use std::rc::Rc;

impl KernelContext<'_, '_> {
    /// Retain only this admitted context's cancellation lineage and absolute
    /// deadline for a worker. Supply the original borrowed lease for leased
    /// execution, or None for serial execution. An active descendant supplies
    /// the effective original backing. No scope, policy or allowance is created.
    ///
    /// Receiver and lineage validation precede control observation/allocation.
    /// Known fixed token storage is admitted before copying and retained by the
    /// move-only view. Successful handoff passes the existing final kernel poll.
    pub fn retain_interruption_view(
        &mut self,
        original_lease: Option<&ExecutionResourceLease<'_>>,
    ) -> Result<ExecutionInterruptionView, ExecutionInterruptionViewDenial> {
        let current = ACTIVE_METER.with(|active| {
            active
                .borrow()
                .last()
                .is_some_and(|meter| Rc::ptr_eq(meter, &self.meter))
        });
        if !current {
            return Err(unrelated_interruption_context());
        }
        let leased = self.meter.borrow().limits.bound_to_lease;
        let lease = interruption_backing_lease(original_lease, leased)?;
        let result = run_invoking_preparation(|| {
            let built = (|| {
                let (count, deadline, accounted) = {
                    let meter = self.meter.borrow();
                    (
                        meter.limits.tokens.len(),
                        meter.limits.deadline,
                        meter.accounting.is_some(),
                    )
                };
                let mut builder =
                    InterruptionViewBuilder::allocate(lease.as_ref(), accounted, count, deadline)?;
                for index in 0..count {
                    let token = self.meter.borrow().limits.tokens[index].clone();
                    builder.push(token)?;
                }
                builder.seal()
            })();
            // No meter borrow survives into a poll or backing disposal.
            self.poll().map_err(|stop| {
                ExecutionInterruptionViewDenial::Request(crate::WorkCeilingDenial::Stopped(stop))
            })?;
            built
        });
        match result {
            Ok(value) => value,
            Err(cause) => Err(ExecutionInterruptionViewDenial::Request(cause)),
        }
    }
}
