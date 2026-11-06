//! Sealed recorder of certified-seed wave acquisition orders: the order the
//! shipped owned dispatcher actually acquired members in. Disarmed, the order is
//! dropped as before.
use std::cell::RefCell;
use std::marker::PhantomData;

thread_local! {
    static ORDERS: RefCell<Option<Vec<Vec<usize>>>> = const { RefCell::new(None) };
}

/// Owned handle of an armed recorder; dropping it, unwinding included, disarms
/// and discards what it recorded.
pub struct SeededDispatchOrdersForTest {
    _thread_bound: PhantomData<*const ()>,
}

impl SeededDispatchOrdersForTest {
    /// The most recent acquisition order recorded on this thread, removed.
    pub fn take_last(&self) -> Option<Vec<usize>> {
        ORDERS.with(|orders| orders.borrow_mut().as_mut().and_then(Vec::pop))
    }
}

impl Drop for SeededDispatchOrdersForTest {
    fn drop(&mut self) {
        ORDERS.with(|orders| *orders.borrow_mut() = None);
    }
}

/// Records every certified-seed acquisition order on this thread until the
/// handle drops; arming starts from an empty record.
pub fn record_seeded_dispatch_for_test() -> SeededDispatchOrdersForTest {
    ORDERS.with(|orders| *orders.borrow_mut() = Some(Vec::new()));
    SeededDispatchOrdersForTest {
        _thread_bound: PhantomData,
    }
}

pub(super) fn record(order: Vec<usize>) {
    ORDERS.with(|orders| {
        if let Some(orders) = orders.borrow_mut().as_mut() {
            orders.push(order);
        }
    });
}
