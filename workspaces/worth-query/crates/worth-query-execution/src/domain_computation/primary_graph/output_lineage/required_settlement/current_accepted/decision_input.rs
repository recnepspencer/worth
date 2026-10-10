//! Changed decision facts or consumed outputs permit another authoritative attempt.
use super::*;

impl AcceptedCurrentCandidate {
    pub(in crate::domain_computation::primary_graph) fn decision_input_changed_at(
        &self,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CurrentAcceptedStop> {
        if !self
            .own_evidence_is_current(runtime, snapshot, admission)
            .map_err(CurrentAcceptedStop::Closure)?
        {
            return Ok(true);
        }
        match ConsumedOutputEvidence::verify_many_with_admission(
            &self.selected.recorded().consumed_outputs,
            owner,
            runtime,
            snapshot,
            selected,
            admission,
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
