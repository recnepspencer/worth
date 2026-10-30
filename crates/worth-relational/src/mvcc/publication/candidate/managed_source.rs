//! Reborrow owned Source observations at the actual consuming candidate basis.
use super::*;
impl PreparedRelationalCommitCandidate {
    pub fn require_managed_source(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observed: &crate::mvcc::ManagedIndexSourceObservation,
    ) -> Result<(), super::super::CompanionPreflightStop> {
        use super::super::CompanionPreflightStop as Stop;
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::CandidateRegistration,
        )
        .map_err(|cause| Stop::Execution(cause.into()))?;
        let mut payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let payload = payload.as_mut().ok_or(Stop::RegistrationChanged)?;
        let condition = super::super::companion::ManagedSourceReadConditions::from_observation(
            observed,
            self.runtime_instance_id,
            &self.branch_id,
            payload.expected_root.id(),
            payload.expected_root.commit_id(),
        )?;
        if let Some(previous) = &payload.managed_source_reads {
            if !previous.same_reserved_image(&condition) {
                return Err(Stop::SelectedSourceMismatch);
            }
        } else {
            payload.managed_source_reads = Some(condition);
        }
        Ok(())
    }
}
