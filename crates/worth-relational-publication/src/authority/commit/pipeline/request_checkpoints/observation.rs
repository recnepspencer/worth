//! Scoped request-stage visit observation for Runtime courts.
use super::CommitStage;
use std::cell::RefCell;
type Observer = Box<dyn FnMut(CommitStage)>;
thread_local! {
    static OBSERVER: RefCell<Option<Observer>> = const { RefCell::new(None) };
}
struct Restore(Option<Observer>);
impl Drop for Restore {
    fn drop(&mut self) {
        OBSERVER.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}
#[doc(hidden)]
pub fn with_stage_observer_for_test<R>(observer: Observer, body: impl FnOnce() -> R) -> R {
    let previous = OBSERVER.with(|slot| slot.replace(Some(observer)));
    let _restore = Restore(previous);
    body()
}
pub(in crate::internal_port::authority::commit::pipeline) fn visit(stage: CommitStage) {
    OBSERVER.with(|slot| {
        if let Some(observer) = slot.borrow_mut().as_mut() {
            observer(stage);
        }
    });
}
