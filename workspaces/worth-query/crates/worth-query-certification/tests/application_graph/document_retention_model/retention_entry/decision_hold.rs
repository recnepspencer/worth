//! A court's in-call memory hold, taken when a retention handler decides.
//!
//! One caller lease funds a whole mutation call. Owners before the handler
//! (principal resolution's native query completion) reserve a transient bound
//! larger than a later owner's charge and release it before the handler runs,
//! so no fixed caller-lease room isolates an owner inside the decision. A
//! court installs a hold instead: the first decision on its thread reserves
//! what the court asks for, after those earlier owners have released, and
//! the court releases it when its call returns.
use std::{any::Any, cell::RefCell};

type Hold = Box<dyn FnOnce() -> Box<dyn Any>>;

thread_local! {
    static PENDING: RefCell<Option<Hold>> = const { RefCell::new(None) };
    static HELD: RefCell<Option<Box<dyn Any>>> = const { RefCell::new(None) };
}

/// Runs `call`; the first retention decision it reaches on this thread takes
/// `hold`, whose result stays held until `call` returns.
pub fn with_decision_hold<T>(
    hold: impl FnOnce() -> Box<dyn Any> + 'static,
    call: impl FnOnce() -> T,
) -> T {
    PENDING.with(|pending| *pending.borrow_mut() = Some(Box::new(hold)));
    let outcome = call();
    let unused = PENDING.with(|pending| pending.borrow_mut().take());
    assert!(
        unused.is_none(),
        "the call never reached a retention decision"
    );
    drop(HELD.with(|held| held.borrow_mut().take()));
    outcome
}

pub(super) fn take_decision_hold() {
    if let Some(hold) = PENDING.with(|pending| pending.borrow_mut().take()) {
        let taken = hold();
        HELD.with(|held| *held.borrow_mut() = Some(taken));
    }
}
