//! Confirm a prepared result without reacquiring an already borrowed Native owner.
use super::*;
use crate::domain_computation::primary_graph::product_operation::document_history::{
    authority_lifetime::HistoryAuthorityTryStop, current_head,
};
use crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitLane;

impl HistoryProducerInvocation<'_> {
    pub(in crate::domain_computation::primary_graph) fn confirm_in_borrowed<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        lane: &WorthQueryApplicationBranchCommitLane,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        value: T,
    ) -> Result<T, HistoryBorrowedReadStop> {
        use HistoryBorrowedReadStop as Stop;
        use WorthQueryHistoryNavigationDenial as Denial;
        let mut value = Some(value);
        let result = self.selection.try_while_live(|| {
            self.callback.try_while_live(|| {
                let Some(coordination) = lane.try_enter() else {
                    return Err(Stop::Busy);
                };
                let original = value.take().expect("one original result handoff");
                match current_head::confirm_in_native(
                    runtime,
                    execution,
                    self.current,
                    Some(self.request),
                    &coordination,
                    native,
                    original,
                ) {
                    Ok(delivered) => Ok(delivered),
                    Err((original, cause)) => {
                        value = Some(original);
                        Err(Stop::Navigation(Denial::Document(cause)))
                    }
                }
            })
        });
        // Neither refused-value cleanup nor public-denial allocation runs under seals.
        match result {
            Ok(Ok(Ok((delivered, None)))) => Ok(delivered),
            Ok(Ok(Ok((_refused, Some(lapse))))) => Err(Stop::Navigation(Denial::Authorization(
                WorthQueryOperationAuthorizationDenial::new(
                    lapse.denial_kind(),
                    "history source query",
                ),
            ))),
            Ok(Ok(Err(stop))) => Err(stop),
            Err(HistoryAuthorityTryStop::Busy) | Ok(Err(HistoryAuthorityTryStop::Busy)) => {
                Err(Stop::Busy)
            }
            _ => Err(Stop::Navigation(Denial::HeadChanged)),
        }
    }
}
