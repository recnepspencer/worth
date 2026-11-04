//! Scoped readonly recording of actual producer publications, without issuing custody.
use super::WorthQueryApplicationCommitReceipt;
use std::{cell::RefCell, marker::PhantomData, rc::Rc};
struct Observer {
    runtime: u64,
    callback: Box<dyn Fn(&WorthQueryApplicationCommitReceipt)>,
}
thread_local! {static OBSERVER:RefCell<Option<Observer>>=const{RefCell::new(None)};}
pub(crate) struct RecordingGuard(PhantomData<Rc<()>>);
impl Drop for RecordingGuard {
    fn drop(&mut self) {
        OBSERVER.with(|slot| {
            slot.borrow_mut().take();
        });
    }
}
pub(crate) fn install(
    runtime: u64,
    callback: Box<dyn Fn(&WorthQueryApplicationCommitReceipt)>,
) -> RecordingGuard {
    OBSERVER.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(
            slot.is_none(),
            "one readonly settlement recorder per thread"
        );
        *slot = Some(Observer { runtime, callback });
    });
    RecordingGuard(PhantomData)
}
pub(super) fn observe(settlement: &WorthQueryApplicationCommitReceipt) {
    OBSERVER.with(|slot| {
        if let Some(observer) = slot.borrow().as_ref()
            && observer.runtime == settlement.runtime_authority().as_u64()
        {
            (observer.callback)(settlement);
        }
    });
}
