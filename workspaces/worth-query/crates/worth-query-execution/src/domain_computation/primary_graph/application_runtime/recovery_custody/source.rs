//! Cold claims join the actual World original before any serving witness escapes.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial;
use worth_runtime_world::facade::WorldProgramActivationHistory;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn admit_recovery_claim_sources(
        &self,
        originals: &WorldProgramActivationHistory,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        for recorded in self
            .runtime
            .dispatch_journal
            .recorded_recovery_claims()
            .map_err(Denial::journal)?
        {
            execution
                .consult()
                .map_err(|cause| Denial::journal(QueryDispatchJournalDenial::Execution(cause)))?;
            let original = originals
                .original_state(recorded.claim.attempt)
                .ok_or_else(|| Denial::journal(QueryDispatchJournalDenial::MalformedFrame))?;
            if original.reserved_commit() != recorded.claim.original
                || original.fate() != worth_foundational::facade::WorldOriginalFate::Published
            {
                return Err(Denial::journal(QueryDispatchJournalDenial::MalformedFrame));
            }
        }
        Ok(())
    }
}
