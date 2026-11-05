//! A scoped readonly recorder distinguishes healthy verification from root reads.
use std::cell::Cell;
thread_local! {
    static READS: Cell<Option<u64>> = const { Cell::new(None) };
}
pub(super) struct Observation;
impl Observation {
    pub(super) fn begin() -> Self {
        READS.with(|reads| assert!(reads.replace(Some(0)).is_none()));
        Self
    }
    pub(super) fn reads(&self) -> u64 {
        READS.with(|reads| reads.get().expect("the observation is installed"))
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        READS.with(|reads| reads.set(None));
    }
}
pub(super) fn record(query: &str) {
    if query == "owned-outputs.producer-model-read.v1" {
        READS.with(|reads| {
            if let Some(count) = reads.get() {
                reads.set(Some(count + 1));
            }
        });
    }
}
