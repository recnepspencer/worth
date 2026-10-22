//! A diagnostic owns its meters instead of inheriting an advancement's bill.
use super::{KernelState, ACTIVE_METER};
use std::{cell::RefCell, rc::Rc};

pub(crate) struct DetachedMeterForTest(Vec<Rc<RefCell<KernelState>>>);

pub(crate) fn detach_meter_for_test() -> DetachedMeterForTest {
    DetachedMeterForTest(ACTIVE_METER.with(|active| std::mem::take(&mut *active.borrow_mut())))
}

impl Drop for DetachedMeterForTest {
    fn drop(&mut self) {
        ACTIVE_METER.with(|active| {
            debug_assert!(active.borrow().is_empty(), "diagnostic meters have settled");
            *active.borrow_mut() = std::mem::take(&mut self.0);
        });
    }
}
