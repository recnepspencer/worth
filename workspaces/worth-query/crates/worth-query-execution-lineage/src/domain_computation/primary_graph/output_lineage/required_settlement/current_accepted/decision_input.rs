//! Changed decision facts or consumed outputs permit another authoritative attempt.
use super::*;

impl AcceptedCurrentCandidate {
    pub fn decision_input_changed_at(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<bool, CurrentAcceptedStop> {
        let origin = self.selected.recorded().computation_source;
        // Native publication can prove that its own effect replaced an input.
        // Its postconditions deliberately cannot compare that old computation.
        // Missing checkpoint evidence carries no such proof.
        if origin != super::super::super::ComputationSourceEvidence::unavailable()
            && origin.certify_current().is_none()
        {
            return Ok(true);
        }
        if !self
            .own_evidence_is_current(request_execution, runtime, basis, selected)
            .map_err(CurrentAcceptedStop::Closure)?
        {
            return Ok(true);
        }
        match ConsumedOutputEvidence::verify_many_with_admission(
            request_execution,
            &self.selected.recorded().consumed_outputs_at(selected),
            owner,
            runtime,
            basis,
            selected,

        ) {
            Ok(answer) => Ok(answer != ConsumedOutputVerification::Current),
            // Missing certification is not evidence of a changed input.
            Err(
                ConsumedOutputVerificationStop::PendingUpstream
                | ConsumedOutputVerificationStop::Unavailable,
            ) => Ok(false),
            Err(stop) => Err(CurrentAcceptedStop::Closure(stop)),
        }
    }
}

mod court_doors_for_test;
