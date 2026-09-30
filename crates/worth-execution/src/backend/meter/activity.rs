use std::{
    cell::RefCell,
    rc::Rc,
    sync::{atomic::Ordering, Arc},
};

use super::{KernelContext, MemoryActivityGuard, PhysicalActivity, RunLimits, ACTIVE_METER};

thread_local! {
    static CERTIFICATION_PHYSICAL: RefCell<Vec<Arc<PhysicalActivity>>> = const { RefCell::new(Vec::new()) };
}

pub(crate) struct CertificationActivityGuard(Arc<PhysicalActivity>);

impl CertificationActivityGuard {
    pub(crate) fn peak_memory(&self) -> u64 {
        self.0.peak_memory.load(Ordering::Acquire)
    }
}

impl Drop for CertificationActivityGuard {
    fn drop(&mut self) {
        CERTIFICATION_PHYSICAL.with(|active| {
            active
                .borrow_mut()
                .pop()
                .expect("balanced certification activity");
        });
    }
}

pub(crate) fn active_certification_physical() -> Option<Arc<PhysicalActivity>> {
    CERTIFICATION_PHYSICAL.with(|active| active.borrow().last().cloned())
}

pub(crate) fn enter_certification_activity() -> CertificationActivityGuard {
    let physical = ACTIVE_METER
        .with(|active| {
            active
                .borrow()
                .last()
                .map(|meter| Arc::clone(&meter.borrow().limits.physical_activity))
        })
        .or_else(active_certification_physical)
        .unwrap_or_else(|| Arc::new(PhysicalActivity::default()));
    CERTIFICATION_PHYSICAL.with(|active| active.borrow_mut().push(Arc::clone(&physical)));
    CertificationActivityGuard(physical)
}

impl KernelContext<'_, '_> {
    pub(crate) fn is_active(&self) -> bool {
        ACTIVE_METER.with(|active| {
            active
                .borrow()
                .last()
                .is_some_and(|current| Rc::ptr_eq(current, &self.meter))
        })
    }

    /// Reconcile the backend's task-order prefix with the canonical tree
    /// prefix. An aggregate task stop may have discarded work later accepted
    /// by canonical settlement; an earlier join may discard accepted tasks.
    pub(crate) fn reconcile_discarded_work(
        &self,
        backend_charged: u64,
        canonical_accepted: u64,
    ) -> bool {
        let meter = self.meter.borrow();
        if backend_charged >= canonical_accepted {
            meter
                .limits
                .add_discarded_work(backend_charged - canonical_accepted);
            true
        } else {
            let restore = canonical_accepted - backend_charged;
            meter
                .limits
                .physical_activity
                .discarded_work
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                    current.checked_sub(restore)
                })
                .is_ok()
        }
    }
}

pub(crate) fn enter_retained_memory(bytes: u64) -> Option<MemoryActivityGuard> {
    let physical = ACTIVE_METER
        .with(|active| {
            active
                .borrow()
                .last()
                .map(|meter| Arc::clone(&meter.borrow().limits.physical_activity))
        })
        .or_else(active_certification_physical)?;
    let current = physical.current_memory.fetch_add(bytes, Ordering::AcqRel) + bytes;
    physical.peak_memory.fetch_max(current, Ordering::AcqRel);
    Some(MemoryActivityGuard(physical, bytes))
}

pub(crate) fn has_active_kernel() -> bool {
    RunLimits::has_parent()
}
