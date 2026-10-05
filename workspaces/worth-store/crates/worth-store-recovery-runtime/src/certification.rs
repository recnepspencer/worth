//! Feature-only exercise of the genuine C8 claim and Store media rejoin.
//! Ordinary recovery never enters this thread-scoped authority.

use std::cell::RefCell;

struct State {
    active: bool,
    pause: Option<Box<dyn FnOnce(worth_store_physical_format::CurrentPhysicalRecordPlacement)>>,
    rejoin_pause: Option<Box<dyn FnOnce()>>,
    before_cleanup_revalidation: Option<Box<dyn FnOnce()>>,
}

thread_local! {
    static STATE: RefCell<State> = const {
        RefCell::new(State {
            active: false,
            pause: None,
            rejoin_pause: None,
            before_cleanup_revalidation: None,
        })
    };
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

/// Installs one change that runs after Store's checkpoint residue gate and
/// before cleanup revalidates its first candidate, and is left in place: the
/// check-then-act race cleanup's own byte-exact revalidation must close.
pub(crate) struct CleanupRevalidationHook;

impl CleanupRevalidationHook {
    pub(crate) fn install(change: impl FnOnce() + 'static) -> Self {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            assert!(
                state.before_cleanup_revalidation.is_none(),
                "nested cleanup revalidation hook"
            );
            state.before_cleanup_revalidation = Some(Box::new(change));
        });
        Self
    }
}

impl Drop for CleanupRevalidationHook {
    fn drop(&mut self) {
        STATE.with(|state| state.borrow_mut().before_cleanup_revalidation = None);
    }
}

pub(crate) fn before_cleanup_revalidation() {
    let hook = STATE.with(|state| state.borrow_mut().before_cleanup_revalidation.take());
    if let Some(hook) = hook {
        hook();
    }
}
