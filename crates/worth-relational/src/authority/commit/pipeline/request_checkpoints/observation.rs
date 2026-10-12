//! Test-local stage rendezvous and consult counting; no production hook.
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
pub(crate) fn with_observer<R>(observer: Observer, body: impl FnOnce() -> R) -> R {
    let previous = OBSERVER.with(|slot| slot.replace(Some(observer)));
    let _restore = Restore(previous);
    body()
}
pub(in crate::authority::commit::pipeline) fn visit(stage: CommitStage) {
    OBSERVER.with(|slot| {
        if let Some(observer) = slot.borrow_mut().as_mut() {
            observer(stage);
        }
    });
}
