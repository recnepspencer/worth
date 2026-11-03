//! A passive observer recognizes the actual issuing meter without retaining it.
use super::{KernelContext, KernelState, ACTIVE_METER};
use std::{cell::RefCell, rc::Weak};

pub(crate) struct ObservationScope(Weak<RefCell<KernelState>>);
impl ObservationScope {
    pub(crate) fn current(context: &KernelContext<'_, '_>) -> Option<Self> {
        ACTIVE_METER.with(|active| {
            let active = active.borrow();
            (active.len() == 1 && std::rc::Rc::ptr_eq(&active[0], &context.meter))
                .then(|| Self(std::rc::Rc::downgrade(&context.meter)))
        })
    }
    pub(crate) fn admits_current(&self) -> bool {
        ACTIVE_METER.with(|active| {
            active
                .borrow()
                .iter()
                .any(|meter| self.0.ptr_eq(&std::rc::Rc::downgrade(meter)))
        })
    }
}
