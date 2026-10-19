//! Fixed stage facts recorded only under the existing scoped execution observer.
use std::cell::{Cell, RefCell};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Stage {
    Published,
    DeliveryStarted,
    Delivered,
    DeliveryWait,
    Ready,
    ReadySuperseded,
    ReadyWorkExhausted,
    ReadySettled,
    OwnPublicationContinuation,
    OwnPublicationReplaced,
    DisclosureExhausted,
    CheckpointWait,
}
thread_local! {
    static STAGES: RefCell<Vec<Stage>> = const { RefCell::new(Vec::new()) };
    static REGISTERED: Cell<bool> = const { Cell::new(false) };
}
pub(super) fn record(stage: Stage) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear);
    }
    STAGES.with(|stages| stages.borrow_mut().push(stage));
}
pub(super) fn take() -> Vec<Stage> {
    STAGES.with(|stages| std::mem::take(&mut *stages.borrow_mut()))
}
fn clear() {
    STAGES.with(|stages| stages.borrow_mut().clear());
    REGISTERED.with(|registered| registered.set(false));
}
