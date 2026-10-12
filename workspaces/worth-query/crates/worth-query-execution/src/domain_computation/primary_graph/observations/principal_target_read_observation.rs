use std::cell::Cell;
thread_local! { static READS: Cell<u64> = const { Cell::new(0) }; }
pub(super) fn record() {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    READS.with(|reads| {
        reads.set(
            reads
                .get()
                .checked_add(1)
                .expect("principal-target read count overflow"),
        );
    });
}
/// Returns native principal-target plan execution attempts on the current thread.
/// Compare counts before and after one host call; this observer is test-only.
#[doc(hidden)]
pub fn principal_target_reads_on_this_thread_for_test() -> u64 {
    READS.with(Cell::get)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::execution_observation::WorthQueryExecutionObservation;
    #[test]
    fn principal_target_read_observation_requires_a_live_listener() {
        assert!(!crate::domain_computation::execution_observation::is_installed());
        let before = principal_target_reads_on_this_thread_for_test();
        record();
        assert_eq!(principal_target_reads_on_this_thread_for_test(), before);
        let listener = WorthQueryExecutionObservation::install_on_this_thread();
        record();
        assert_eq!(principal_target_reads_on_this_thread_for_test(), before + 1);
        drop(listener);
        record();
        assert_eq!(principal_target_reads_on_this_thread_for_test(), before + 1);
    }
}
