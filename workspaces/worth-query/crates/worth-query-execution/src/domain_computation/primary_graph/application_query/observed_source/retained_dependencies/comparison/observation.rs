//! Passive receiver-owned observations of the complete native comparison meter.
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use std::sync::{Arc, Mutex};
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct RequiredSetComparisonObservations {
    reports: Arc<Mutex<Reports>>,
}
#[derive(Default)]
struct Reports {
    registered: bool,
    work: Vec<u64>,
}
impl RequiredSetComparisonObservations {
    pub(in crate::domain_computation::primary_graph) fn record(&self, work: u64) {
        if !crate::domain_computation::execution_observation::is_installed() {
            return;
        }
        let owner = &self.reports;
        let mut reports = owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !reports.registered {
            let weak = Arc::downgrade(owner);
            crate::domain_computation::execution_observation::register_owned_cleanup(move || {
                if let Some(owner) = weak.upgrade() {
                    *owner
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Reports::default();
                }
            });
            reports.registered = true;
        }
        reports.work.push(work);
    }
}
impl<S> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Drains actual complete-comparison charges from this receiver under an owned listener.
    /// Neither a query receipt nor an execution report substitutes for this meter.
    #[doc(hidden)]
    pub fn take_required_set_comparison_work_for_test(&self) -> Vec<u64> {
        let mut reports = self
            .required_set_comparison_observations
            .reports
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::take(&mut reports.work)
    }
}
