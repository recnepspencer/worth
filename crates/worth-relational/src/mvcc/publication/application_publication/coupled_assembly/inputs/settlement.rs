//! Bind exact settlement to the Runtime-owned candidate and its lazy pin.
use super::*;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::input_issuance::settlement::compare_retained_input;

type CandidatePin = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Compare Native consumption identity and payload between `old_parent` (at
    /// `old_parent_pin`) and `replacement` (at the current candidate pin). If
    /// equal, settle `consumer`'s retained `RequiredOutputInputs` for that route
    /// to `replacement`'s meaning and return `Ok(true)`. Otherwise return
    /// `Ok(false)` without mutating `consumer`.
    pub fn settle_output_input(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        consumer: &RelationalRequiredOutputInput,
        old_parent_pin: &CandidatePin,
        old_parent: &RelationalRequiredOutputInput,
        replacement: &RelationalRequiredOutputInput,
    ) -> Result<bool, TransactionCommitError> {
        let issuance = self.input_issuance.borrow().as_ref().cloned();
        let update = compare_retained_input(
            execution,
            self.identity,
            issuance.as_ref(),
            &self.parts,
            &self.mutation.outcome.created_entities,
            consumer,
            old_parent_pin,
            old_parent,
            replacement,
            &mut || self.pin_source(execution),
        )?;
        let Some(update) = update else {
            return Ok(false);
        };
        update.settle(&mut self.parts);
        Ok(true)
    }
}
