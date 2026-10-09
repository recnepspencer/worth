//! Counts actual installed-source reads and World branch observation contacts.
use std::cell::Cell;
thread_local! { static READS: Cell<u64> = const { Cell::new(0) }; }

pub(crate) fn record_read() {
    READS.with(|reads| {
        reads.set(
            reads
                .get()
                .checked_add(1)
                .expect("source read count overflow"),
        )
    });
}

/// Source and World branch reads, including principal, projection and revalidation reads.
/// A refusal probe must pair this count with an admitted control at the same door.
pub fn installed_source_reads_on_this_thread_for_test() -> u64 {
    READS.with(Cell::get)
        + worth_runtime_world::facade::world_branch_reads_on_this_thread_for_test()
}
