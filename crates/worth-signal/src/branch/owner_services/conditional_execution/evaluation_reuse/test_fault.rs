use std::cell::Cell;

thread_local! {
    static PANIC_AFTER_TRANSITION_APPLY: Cell<bool> = const { Cell::new(false) };
}

pub(crate) fn arm_panic_after_transition_apply() {
    PANIC_AFTER_TRANSITION_APPLY.with(|armed| armed.set(true));
}

pub(crate) fn panic_after_transition_apply_if_armed() {
    PANIC_AFTER_TRANSITION_APPLY.with(|armed| {
        if armed.replace(false) {
            panic!("injected panic after conditional transition application");
        }
    });
}
