//! Thread-scoped read-only contacts; this recorder grants no mutation standing.
use super::{CurrentOwnershipContactsForTest, CurrentPublicationRoster};
use std::cell::RefCell;
type Observer = Box<dyn Fn(CurrentOwnershipContactsForTest)>;
thread_local! {
    static OBSERVER: RefCell<Option<Observer>> = const { RefCell::new(None) };
}
pub(super) struct Guard(Option<Observer>);
impl Drop for Guard {
    fn drop(&mut self) {
        OBSERVER.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}
pub(super) fn install(observer: Observer) -> Guard {
    Guard(OBSERVER.with(|slot| slot.replace(Some(observer))))
}
pub(super) fn record(
    current: &CurrentPublicationRoster,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
) {
    OBSERVER.with(|slot| {
        if let Some(observer) = slot.borrow().as_ref() {
            observer(CurrentOwnershipContactsForTest::from_current(
                current, basis,
            ));
        }
    });
}

/// Sample only for an installed recorder and an authentic accounted request.
pub(super) fn work(context: &worth_execution::MapKernelContext<'_, '_>) -> Option<u64> {
    OBSERVER
        .with(|slot| slot.borrow().is_some())
        .then(|| context.has_accounting().then(|| context.charged_work()))
        .flatten()
}
