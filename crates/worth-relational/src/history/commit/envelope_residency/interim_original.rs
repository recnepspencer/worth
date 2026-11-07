//! Lend an actual verified original's weak position to its admitted image cell.
use super::*;
use crate::durability::checkpoint_admission::{
    CheckpointAdmissionBindingMismatch, CheckpointAdmittedCommit,
    RelationalCheckpointAdmissionRefusal as Refusal,
};

impl RelationalCommitEnvelopeResidency {
    pub(crate) fn bind_interim_original(
        &self,
        admitted: &CheckpointAdmittedCommit,
        original: &PositionedCanonicalCommit,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        let index = admitted.index();
        let work = index
            .commit
            .branch_id
            .0
            .len()
            .checked_add(index.commit.parents.len())
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(1))
            .ok_or(Refusal::CapacityOverflow)?;
        crate::durability::artifact_payload_emission::request::account_work(request, work as u64)
            .map_err(crate::durability::checkpoint_admission::native_stop)?;
        let interim = match admitted.scope() {
            worth_foundational::facade::CheckpointAdmissionScope::Interim { .. } => true,
            worth_foundational::facade::CheckpointAdmissionScope::Home { .. } => false,
        };
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(failure) = state.damage {
            return Err(crate::durability::checkpoint_admission::native_stop(
                failure.into(),
            ));
        }
        let selected = match &state.backing {
            EnvelopeBacking::SelectedCheckpoint { entry, .. } => entry.as_ref() == index,
            EnvelopeBacking::Memory { .. } | EnvelopeBacking::Journal { .. } => false,
        };
        if !interim
            || !selected
            || original.position() != index.position
            || original.commit != index.commit
            || original.schema_version != index.schema_version
        {
            return Err(Refusal::BindingMismatch(
                CheckpointAdmissionBindingMismatch::NativeOrigin,
            ));
        }
        // Only Weak custody: the immutable capture origin remains the payload
        // owner. Its release makes subsequent ordinary demands cold again.
        state.position = Some(RetainedCanonicalPosition::from_positioned(original));
        Ok(())
    }
}
