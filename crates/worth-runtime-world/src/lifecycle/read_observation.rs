//! Counts actual World branch observation and head-validation reads on this thread.
use std::{cell::Cell, marker::PhantomData, rc::Rc};
thread_local! {
    static READS: Cell<u64> = const { Cell::new(0) };
    static LISTENERS: Cell<usize> = const { Cell::new(0) };
}

/// Owns World read observation on this thread without granting World authority.
#[must_use = "the observation ends when its guard is dropped"]
pub struct RuntimeWorldReadObservation {
    _thread: PhantomData<Rc<()>>,
}
impl RuntimeWorldReadObservation {
    /// Count real owner reads until the final listener on this thread drops.
    pub fn install_on_this_thread() -> Self {
        LISTENERS.with(|listeners| {
            listeners.set(
                listeners
                    .get()
                    .checked_add(1)
                    .expect("World listener count overflow"),
            );
        });
        Self {
            _thread: PhantomData,
        }
    }
}
impl Drop for RuntimeWorldReadObservation {
    fn drop(&mut self) {
        LISTENERS.with(|listeners| listeners.set(listeners.get() - 1));
    }
}

pub(super) fn record_read() {
    if LISTENERS.with(Cell::get) == 0 {
        return;
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_owned_thread_listeners_record_world_reads() {
        let before = world_branch_reads_on_this_thread_for_test();
        record_read();
        assert_eq!(world_branch_reads_on_this_thread_for_test(), before);
        let outer = RuntimeWorldReadObservation::install_on_this_thread();
        let inner = RuntimeWorldReadObservation::install_on_this_thread();
        record_read();
        drop(outer);
        record_read();
        drop(inner);
        record_read();
        assert_eq!(world_branch_reads_on_this_thread_for_test(), before + 2);
    }
}
