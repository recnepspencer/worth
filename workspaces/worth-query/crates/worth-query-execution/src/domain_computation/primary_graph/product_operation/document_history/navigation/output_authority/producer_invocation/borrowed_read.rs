//! Confirm a prepared result without reacquiring an already borrowed Native owner.
use super::*;
use crate::domain_computation::primary_graph::product_operation::document_history::{
    authority_lifetime::HistoryAuthorityTryStop,
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
                let original = value.take().expect("one original result handoff");
                match self
                    .read_control
                    .confirm_in_borrowed_preserving(execution, runtime, lane, native, original)
                {
                    Ok(delivered) => Ok(delivered),
                    Err((original, cause)) => {
                        value = Some(original);
                        Err(cause)
                    }
                }
            })
        });
        // Neither refused-value cleanup nor public-denial allocation runs under seals.
        match result {
            Ok(Ok(Ok(delivered))) => Ok(delivered),
            Ok(Ok(Err(crate::domain_computation::primary_graph::product_operation::current_read::CurrentReadDenial::Busy))) => Err(Stop::Busy),
            Ok(Ok(Err(cause))) => Err(Stop::Navigation(super::read_confirmation::read_denial(cause, "history source query"))),
            Err(HistoryAuthorityTryStop::Busy) | Ok(Err(HistoryAuthorityTryStop::Busy)) => {
                Err(Stop::Busy)
            }
            _ => Err(Stop::Navigation(Denial::HeadChanged)),
        }
    }
}
