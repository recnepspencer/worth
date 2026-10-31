//! One read-only holder of the actual completed composite source.
use super::{HistoryModelIdentity, WorthQueryHistoryStep};
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
use std::sync::Arc;

#[must_use = "retain this historical source or close its holder"]
pub struct WorthQueryHistoryPreview {
    pub(in crate::domain_computation::primary_graph) step: WorthQueryHistoryStep,
    pub(in crate::domain_computation::primary_graph) model: HistoryModelIdentity,
    pub(in crate::domain_computation::primary_graph) source:
        Arc<WorthQueryApplicationReadObservation>,
}

impl WorthQueryHistoryPreview {
    /// Descriptive evidence from the admitted opening, not fresh READ permission.
    pub fn step(&self) -> &WorthQueryHistoryStep {
        &self.step
    }

    /// Release only this holder's original source lease.
    pub fn close(self) {}
}

impl std::fmt::Debug for WorthQueryHistoryPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryPreview")
            .finish_non_exhaustive()
    }
}
