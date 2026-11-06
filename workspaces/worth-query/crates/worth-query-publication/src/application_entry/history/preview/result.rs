//! Preserve the actual query result and custody without exposing retained-selection doors.
//! ```compile_fail,E0599
//! use worth_query_publication::facade::application_entry::WorthQueryHistoryPreviewResult;
//! fn extract<Q, R>(result: WorthQueryHistoryPreviewResult<Q, R>) { result.observed_sources(); }
//! ```
//! Read the admitted rows and release their original custody.
//! ```
//! use worth_query_publication::facade::application_entry::WorthQueryHistoryPreviewResult;
//! fn count<Q, R>(result: WorthQueryHistoryPreviewResult<Q, R>) -> usize {
//!     let count = result.rows().len(); result.close(); count
//! }
//! ```
use crate::domain_computation::{
    WorthQueryPublishedApplicationBasis, WorthQueryPublishedApplicationResult,
};
use worth_query_execution::facade::product::WorthQueryProductBranchReadIdentity;

pub struct WorthQueryHistoryPreviewResult<Q, R> {
    pub(super) original: WorthQueryPublishedApplicationResult<Q, R>,
    pub(super) source: WorthQueryProductBranchReadIdentity,
    pub(super) root_posture: Option<crate::application_entry::WorthQueryOutputSettlementPosture>,
}
impl<Q, R> WorthQueryHistoryPreviewResult<Q, R> {
    /// Actual recorded root posture, disclosed with this fresh historical read.
    /// This is neither current whole-tree readiness nor producer re-execution.
    pub fn required_root_posture(
        &self,
    ) -> Option<crate::application_entry::WorthQueryOutputSettlementPosture> {
        self.root_posture
    }
    pub fn rows(&self) -> &[R] {
        self.original.rows()
    }
    pub fn source(&self) -> &WorthQueryProductBranchReadIdentity {
        &self.source
    }
    pub fn basis(&self) -> WorthQueryPublishedApplicationBasis {
        self.original.receipt().inspect().basis().clone()
    }
    pub fn close(self) {}
}
impl<Q, R> std::fmt::Debug for WorthQueryHistoryPreviewResult<Q, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryPreviewResult")
            .finish_non_exhaustive()
    }
}
