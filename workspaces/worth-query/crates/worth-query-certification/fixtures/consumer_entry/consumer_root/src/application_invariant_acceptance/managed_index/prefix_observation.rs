//! Receiver-owned observations for each independently installed prefix provider.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};
use worth_query_host::facade::primary_graph::WorthQueryManagedIndexSourceSlot;
pub(crate) const OBSERVED_COORDINATES: usize = 8;
#[derive(Default)]
pub struct Calls {
    #[cfg(test)]
    pub omit_refinement: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub reject_refinement: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub after_refinement: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    #[cfg(test)]
    pub predicate_regions: AtomicUsize,
    #[cfg(test)]
    pub(crate) cost_stages: super::prefix_cost_stages::CostStages,
    #[cfg(test)]
    pub query_read_first: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub query_region_reads: AtomicUsize,
    #[cfg(test)]
    pub summary_repetitions: AtomicUsize,
    #[cfg(test)]
    pub inspect_text: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub wrong_text_binding: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub text_reads: AtomicUsize,
    #[cfg(test)]
    pub after_text: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    #[cfg(test)]
    pub summary_read_first: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub(crate) first_source: Mutex<Option<WorthQueryManagedIndexSourceSlot>>,
    #[cfg(test)]
    pub(crate) after_second_match: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    #[cfg(test)]
    pub(crate) repeat_candidates: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    pub(crate) foreign_candidate: Mutex<Option<WorthQueryManagedIndexSourceSlot>>,
    pub matched_sources: AtomicUsize,
    pub successful_matches: AtomicUsize,
    pub summary_comparisons: AtomicUsize,
    #[cfg(test)]
    pub(crate) structure_fault: Mutex<super::structure_fault::Fault>,
    pub capacity: AtomicUsize,
    pub builds: AtomicUsize,
    pub probe_validations: AtomicUsize,
    pub updates: AtomicUsize,
    pub canonical_rows: AtomicUsize,
    // Consumer fixture observation, invoked only after a genuine first node write.
    pub after_first_node: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    pub canonical_coordinates: Mutex<[(u32, u64, u32); OBSERVED_COORDINATES]>,
}
impl Calls {
    pub fn totals(&self) -> (usize, usize, usize) {
        (
            self.capacity.load(Ordering::SeqCst),
            self.builds.load(Ordering::SeqCst),
            self.probe_validations.load(Ordering::SeqCst),
        )
    }
}
