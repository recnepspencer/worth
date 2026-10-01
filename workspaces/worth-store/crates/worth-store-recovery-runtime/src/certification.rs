//! Feature-only exercise of the genuine C8 claim and Store media rejoin.
//! Ordinary recovery never enters this thread-scoped authority.

use std::cell::RefCell;

struct State {
    active: bool,
    pause: Option<Box<dyn FnOnce(worth_store_physical_format::CurrentPhysicalRecordPlacement)>>,
    rejoin_pause: Option<Box<dyn FnOnce()>>,
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State { active: false, pause: None, rejoin_pause: None }) };
}

pub(crate) struct CertificationScope;

impl CertificationScope {
    pub(crate) fn enter(
        pause: impl FnOnce(worth_store_physical_format::CurrentPhysicalRecordPlacement) + 'static,
        rejoin_pause: impl FnOnce() + 'static,
    ) -> Self {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            assert!(!state.active, "nested custody certification");
            state.active = true;
            state.pause = Some(Box::new(pause));
            state.rejoin_pause = Some(Box::new(rejoin_pause));
        });
        Self
    }
}

impl Drop for CertificationScope {
    fn drop(&mut self) {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.active = false;
            state.pause = None;
            state.rejoin_pause = None;
        });
    }
}

pub(crate) fn enabled() -> bool {
    STATE.with(|state| state.borrow().active)
}

pub(crate) fn pause_after_claim(
    descriptor: worth_store_physical_format::CurrentPhysicalRecordPlacement,
) {
    let pause = STATE.with(|state| state.borrow_mut().pause.take());
    if let Some(pause) = pause {
        pause(descriptor);
    }
}

pub(crate) fn take_rejoin_pause() -> Box<dyn FnOnce()> {
    STATE
        .with(|state| state.borrow_mut().rejoin_pause.take())
        .expect("certification rejoin pause is installed")
}
