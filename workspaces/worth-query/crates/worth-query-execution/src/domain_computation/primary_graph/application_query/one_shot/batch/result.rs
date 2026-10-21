use super::super::super::{batch::PlannedBatchItem, WorthQueryApplicationQueryBatchAdmission};
use super::WorthQueryApplicationOneShotResult;
/// Successful native read paired with its consumed installed schedule slot.
pub struct WorthQueryApplicationBatchResult<Query, Result> {
    result: WorthQueryApplicationOneShotResult<Query, Result>,
    item: PlannedBatchItem,
}
impl<Q, R> WorthQueryApplicationBatchResult<Q, R> {
    pub(super) fn issued(
        result: WorthQueryApplicationOneShotResult<Q, R>,
        item: PlannedBatchItem,
    ) -> Self {
        Self { result, item }
    }
    pub fn result(&self) -> &WorthQueryApplicationOneShotResult<Q, R> {
        &self.result
    }
    pub fn into_result(self) -> WorthQueryApplicationOneShotResult<Q, R> {
        self.result
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn into_result_for(
        self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> Option<WorthQueryApplicationOneShotResult<Q, R>> {
        self.item.belongs_to(batch).then_some(self.result)
    }
}
