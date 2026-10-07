//! Test faults enter only after the real registry guard has been acquired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Door {
    Fork,
    Enter,
    Begin,
    Remove,
    Activate,
    Head,
}
thread_local! { static ARMED: std::cell::Cell<Option<Door>> = const { std::cell::Cell::new(None) }; }
pub(super) fn arm(door: Door) {
    ARMED.with(|armed| {
        assert!(armed.get().is_none());
        armed.set(Some(door));
    });
}
pub(super) fn after_acquisition(door: Door) {
    if ARMED.with(|armed| armed.get() == Some(door)) {
        ARMED.with(|armed| armed.set(None));
        panic!("registry guard fault: {door:?}");
    }
}
