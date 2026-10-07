//! Certification observation of real, synchronous one-shot read kernel entry.

use std::cell::Cell;

thread_local! {
    static KERNEL_ENTRIES: Cell<u64> = const { Cell::new(0) };
}

pub(super) fn record_kernel_entry() {
    KERNEL_ENTRIES.with(|entries| {
        entries.set(
            entries
                .get()
                .checked_add(1)
                .expect("query entry counter overflow"),
        );
    });
}

/// Includes attempted reads which later deny. Admission and projection-only
/// activity are outside this measurement; other threads have independent counts.
#[doc(hidden)]
pub fn query_read_kernel_entries_on_this_thread_for_test() -> u64 {
    KERNEL_ENTRIES.with(Cell::get)
}
