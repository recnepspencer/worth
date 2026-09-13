//! One-shot thread-local failure immediately after actual destination insertion.

use std::cell::Cell;

thread_local! {
    static ARMED: Cell<bool> = const { Cell::new(false) };
}

pub(crate) struct InstallationUnwind;

pub(crate) fn arm() -> InstallationUnwind {
    assert!(!ARMED.replace(true));
    InstallationUnwind
}

impl Drop for InstallationUnwind {
    fn drop(&mut self) {
        ARMED.set(false);
    }
}

pub(super) fn after_installed() {
    if ARMED.replace(false) {
        panic!("injected unwind after actual registry insertion");
    }
}
