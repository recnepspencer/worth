use std::cell::Cell;
thread_local! { static READS: Cell<u64> = const { Cell::new(0) }; }
pub(super) fn record() {
    READS.with(|reads| reads.set(reads.get() + 1));
}
/// Returns native principal-target plan execution attempts on the current thread.
/// Compare counts before and after one host call; this observer is test-only.
#[doc(hidden)]
pub fn principal_target_reads_on_this_thread_for_test() -> u64 {
    READS.with(Cell::get)
}
