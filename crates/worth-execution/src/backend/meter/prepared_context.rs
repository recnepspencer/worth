use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::authority::ExecutionResourceLease;

use super::{activity, KernelMeter, PhysicalActivity, RunLimits, ACTIVE_METER};

/// A prepared run may retain memory across domain work, but must return to
/// exactly the request context that prepared it. Keeping the Rc also prevents
/// a replaced parent meter from reusing the same address.
pub(crate) struct PreparedMeterContext {
    parent: Option<Rc<RefCell<KernelMeter>>>,
    certification: Option<Arc<PhysicalActivity>>,
    physical: Arc<PhysicalActivity>,
    physical_cell_charged: bool,
}

impl PreparedMeterContext {
    pub(crate) fn capture(limits: &RunLimits) -> Self {
        let parent = ACTIVE_METER.with(|active| active.borrow().last().cloned());
        let certification = if parent.is_none() {
            activity::active_certification_physical()
        } else {
            None
        };
        Self {
            parent,
            certification,
            physical: Arc::clone(&limits.physical_activity),
            physical_cell_charged: limits.physical_cell_charged,
        }
    }

    pub(crate) fn matches_current_parent(&self) -> bool {
        let parent_matches = ACTIVE_METER.with(|active| {
            let active = active.borrow();
            match (self.parent.as_ref(), active.last()) {
                (None, None) => true,
                (Some(expected), Some(current)) => Rc::ptr_eq(expected, current),
                _ => false,
            }
        });
        if !parent_matches {
            return false;
        }
        if self.parent.is_some() {
            return true;
        }
        let current = activity::active_certification_physical();
        match (self.certification.as_ref(), current.as_ref()) {
            (None, None) => true,
            (Some(expected), Some(current)) => Arc::ptr_eq(expected, current),
            _ => false,
        }
    }

    pub(crate) fn dispatch_limits(
        &self,
        lease: &ExecutionResourceLease<'_>,
        serial: bool,
    ) -> RunLimits {
        RunLimits::for_run_with_activity(
            Some(lease),
            serial,
            Some(Arc::clone(&self.physical)),
            self.physical_cell_charged,
        )
    }
}
