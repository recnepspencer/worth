use super::{
    WorthQueryApplicationOneShotResult, WorthQueryApplicationQueryBatchAdmission,
    WorthQueryApplicationQueryBatchMemory,
};

/// An executor-issued result together with its original shared-loan row custody.
/// It cannot be assembled from an ordinary result and an unrelated memory claim.
/// `result()` borrows without releasing custody; `into_parts()` consumes the seal
/// for ordinary staging. Managed refresh accepts the intact seal and checks its
/// original batch before exposing either part.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryApplicationBatchResult, WorthQueryApplicationOneShotResult,
///     WorthQueryApplicationQueryBatchMemory,
/// };
/// fn forge<Q, R>(result: WorthQueryApplicationOneShotResult<Q, R>,
///                claim: WorthQueryApplicationQueryBatchMemory) {
///     let _ = WorthQueryApplicationBatchResult { result, claim };
/// }
/// ```
pub struct WorthQueryApplicationBatchResult<Query, Result> {
    result: WorthQueryApplicationOneShotResult<Query, Result>,
    claim: WorthQueryApplicationQueryBatchMemory,
}

impl<Query, Result> WorthQueryApplicationBatchResult<Query, Result> {
    pub(super) fn issued(
        result: WorthQueryApplicationOneShotResult<Query, Result>,
        claim: WorthQueryApplicationQueryBatchMemory,
    ) -> Self {
        Self { result, claim }
    }

    pub fn result(&self) -> &WorthQueryApplicationOneShotResult<Query, Result> {
        &self.result
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationOneShotResult<Query, Result>,
        WorthQueryApplicationQueryBatchMemory,
    ) {
        (self.result, self.claim)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn into_parts_for(
        self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> Option<(
        WorthQueryApplicationOneShotResult<Query, Result>,
        WorthQueryApplicationQueryBatchMemory,
    )> {
        self.claim.belongs_to(batch).then(|| self.into_parts())
    }
}
