//! Exact conditional executions enter the actual shared diagnostics owner.
use super::*;
use crate::diagnostics::conditional_history::ConditionalHistoryReference;
impl BridgeDiagnosticsFacade {
    pub(crate) fn retain_conditional_history(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        decision: &worth_signal::facade::SignalConditionalDecisionEvidence,
    ) -> Result<ConditionalHistoryReference, worth_execution::WorkCeilingDenial> {
        self.state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .conditional_history
            .capture(
                execution,
                decision,
                self.config.records_enabled,
                self.config.route_record_limit,
            )
    }
    /// Selects the owner's real retention policy before any retained history.
    #[doc(hidden)]
    pub fn configure_conditional_history_for_test(&self, enabled: bool, limit: usize) -> bool {
        self.state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .conditional_history
            .configure_for_test(enabled, limit)
    }
}
