use super::*;

impl AcceptedCurrentCandidate {
    pub(super) fn own_evidence_is_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, ConsumedOutputVerificationStop> {
        // An old dependency set cannot constrain a consumer whose own native
        // evidence has changed. Missing evidence cannot reject the old edges.
        // This only requests disclosure, never Current.
        admission
            .charge_external_work(3)
            .map_err(|_| ConsumedOutputVerificationStop::WorkExhausted)?;
        let facts = self
            .selected
            .recorded()
            .observed_source_facts()
            .and_then(|facts| facts.for_comparison());
        let (Some(facts), Some(witness)) = (facts, self.selected.native_output_witness()) else {
            return Ok(true);
        };
        let count = self
            .selected
            .completed_handler_fact_count()
            .unwrap_or(facts.len());
        let Some(handler_facts) = facts.get(..count) else {
            return Ok(true);
        };
        // Handler evidence decides whether the old edges still apply. Source
        // query stamps are a suffix whose meaning is checked by input equality.
        match ConsumedOutputEvidence::own_evidence_is_current(
            execution,
            handler_facts,
            &self.selected.recorded().consumed_outputs,
            witness,
            runtime,
            snapshot,
            admission,
        ) {
            Ok(false) => Ok(false),
            Ok(true) | Err(ConsumedOutputVerificationStop::Unavailable) => Ok(true),
            Err(stop) => Err(stop),
        }
    }
}
