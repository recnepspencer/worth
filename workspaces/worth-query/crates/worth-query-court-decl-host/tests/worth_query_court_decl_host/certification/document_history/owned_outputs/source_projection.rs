//! A scoped readonly recorder distinguishes healthy verification from root reads.
use std::cell::Cell;
thread_local! {
    static PUBLIC_MEMBER_READS: Cell<u64> = const { Cell::new(0) };
    static MEMBER_READS: Cell<u64> = const { Cell::new(0) };
    static DISCOVERY_READS: Cell<u64> = const { Cell::new(0) };
    static READS: Cell<Option<u64>> = const { Cell::new(None) };
}
pub(super) struct Observation;
impl Observation {
    pub(super) fn begin() -> Self {
        READS.with(|reads| assert!(reads.replace(Some(0)).is_none()));
        DISCOVERY_READS.with(|reads| reads.set(0));
        MEMBER_READS.with(|reads| reads.set(0));
        PUBLIC_MEMBER_READS.with(|reads| reads.set(0));
        Self
    }
    pub(super) fn public_member_reads(&self) -> u64 {
        PUBLIC_MEMBER_READS.with(Cell::get)
    }
    pub(super) fn protected_member_reads(&self) -> u64 {
        MEMBER_READS.with(Cell::get)
    }
    pub(super) fn discovery_reads(&self) -> u64 {
        DISCOVERY_READS.with(Cell::get)
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
    if query == "owned-outputs.member-public-query.v1" {
        PUBLIC_MEMBER_READS.with(|reads| reads.set(reads.get() + 1));
    }
    if query == "owned-outputs.member-read.v1" {
        MEMBER_READS.with(|reads| reads.set(reads.get() + 1));
    }
    if query == "owned-outputs.first-discovery-query.v1" {
        DISCOVERY_READS.with(|reads| reads.set(reads.get() + 1));
    }
    if query == "owned-outputs.producer-model-read.v1" {
        READS.with(|reads| {
            if let Some(count) = reads.get() {
                reads.set(Some(count + 1));
            }
        });
    }
}
