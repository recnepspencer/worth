use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiPersistentIndexTestWork {
    comparison_steps: usize,
    lookup_probes: usize,
    iterated_entries: usize,
}

impl UiPersistentIndexTestWork {
    pub(crate) const fn comparison_steps(self) -> usize {
        self.comparison_steps
    }
    pub(crate) const fn lookup_probes(self) -> usize {
        self.lookup_probes
    }

    pub(crate) const fn iterated_entries(self) -> usize {
        self.iterated_entries
    }
}

thread_local! {
    static COMPARISON_STEPS: Cell<usize> = const { Cell::new(0) };
    static OBSERVE_ALL: Cell<bool> = const { Cell::new(false) };
    static LOOKUP_PROBES: Cell<usize> = const { Cell::new(0) };
    static ITERATED_ENTRIES: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn observe_comparison_step() {
    if OBSERVE_ALL.get() {
        COMPARISON_STEPS.set(COMPARISON_STEPS.get().saturating_add(1));
    }
}

pub(super) fn observe_lookup(_map: *const (), probes: usize) {
    if OBSERVE_ALL.get() {
        LOOKUP_PROBES.set(LOOKUP_PROBES.get().saturating_add(probes));
    }
}

pub(super) fn observe_iteration(observed: bool) {
    if observed {
        ITERATED_ENTRIES.set(ITERATED_ENTRIES.get().saturating_add(1));
    }
}

pub(super) fn reset_all_test_work() {
    COMPARISON_STEPS.set(0);
    OBSERVE_ALL.set(true);
    LOOKUP_PROBES.set(0);
    ITERATED_ENTRIES.set(0);
}

pub(super) fn observes(_map: *const ()) -> bool {
    OBSERVE_ALL.get()
}

pub(crate) fn test_work() -> UiPersistentIndexTestWork {
    UiPersistentIndexTestWork {
        comparison_steps: COMPARISON_STEPS.get(),
        lookup_probes: LOOKUP_PROBES.get(),
        iterated_entries: ITERATED_ENTRIES.get(),
    }
}
