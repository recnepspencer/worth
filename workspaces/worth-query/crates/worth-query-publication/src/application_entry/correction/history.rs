//! Bounded newest-first correction history under the current request.
use super::*;
pub use worth_query_execution::facade::primary_graph::{
    WorthQueryCorrectionHistoryDenial, WorthQueryCorrectionHistoryRecord,
    WorthQueryCorrectionHistoryState,
};

/// The owning boundary that refused a correction history listing.
#[derive(Debug)]
pub enum WorthQueryApplicationCorrectionHistoryDenial {
    ExecutionRequest(WorthQueryAdvancementDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    History(WorthQueryCorrectionHistoryDenial),
}

impl<S: ApplicationSchema> WorthQueryApplicationCorrections<'_, '_, '_, S> {
    /// Lists at most `limit` retained authored writes this principal may
    /// correct on the request's branch, newest first, with reversal state.
    ///
    /// Each entry was admitted by the same source selection an undo performs;
    /// selecting its handle later rechecks disclosure and ancestry afresh.
    pub fn history(
        &self,
        limit: NonZeroUsize,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> Result<Vec<WorthQueryCorrectionHistoryRecord>, WorthQueryApplicationCorrectionHistoryDenial>
    {
        use WorthQueryApplicationCorrectionHistoryDenial as Denial;
        self.application
            .with_application_advancement(self.scope, |phase| {
                let target = self
                    .application
                    .on_branch(self.branch)
                    .select(&phase)
                    .map_err(Denial::ProductSelection)?;
                target
                    .correction_history(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        &phase,
                        self.principal,
                        self.scope,
                        limit,
                        limits.maximum_ancestors,
                        limits.maximum_disclosure_results,
                    )
                    .map_err(Denial::History)
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
