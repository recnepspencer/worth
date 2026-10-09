//! Counts actual World branch observation and head-validation reads on this thread.
use std::cell::Cell;
thread_local! { static READS: Cell<u64> = const { Cell::new(0) }; }

pub(super) fn record_read() {
    READS.with(|reads| {
        reads.set(
            reads
                .get()
                .checked_add(1)
                .expect("World read count overflow"),
        )
    });
}

/// Branch observation and publication head-validation reads on this thread.
pub fn world_branch_reads_on_this_thread_for_test() -> u64 {
    READS.with(Cell::get)
}
