//! Nonwaiting access to the original selected directory slot.
use super::*;
use worth_relational_storage::internal_port::durability::medium::checkpoint_acknowledgement_denial::RelationalCheckpointAcknowledgementDenial as Ack;
use worth_relational_storage::internal_port::storage::data::directory_source_edge::NativeDirectoryRetirementHandoff;
impl RelationalCheckpointManifestSelection {
    fn try_source_state(&self) -> Result<std::sync::MutexGuard<'_, CheckpointSelectionState>, Ack> {
        match self.selected.try_lock() {
            Ok(state) => Ok(state),
            Err(std::sync::TryLockError::Poisoned(error)) => Ok(error.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => Err(Ack::Busy),
        }
    }
    pub fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Ack> {
        let (original, naming_dir, admission) = {
            let state = self.try_source_state()?;
            (
                state
                    .directory
                    .as_ref()
                    .and_then(NativeCheckpointDirectoryEdge::snapshot),
                state
                    .naming_directory
                    .as_ref()
                    .and_then(NativeCheckpointDirectoryEdge::snapshot),
                state.admission.clone(),
            )
        };
        if let Some(admission) = admission {
            admission.preflight_retirement(source)?;
        }
        if let Some(original) = original {
            original.preflight_retirement(source)?;
        }
        if let Some(naming_dir) = naming_dir {
            naming_dir.preflight_retirement(source)?;
        }
        Ok(())
    }
    pub fn loan_source_retirement(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        let mut state = self.try_source_state()?;
        if let Some(admission) = &mut state.admission
            && !admission.loan_source_retirement(handoff)?
        {
            state.admission_loan = true;
            state.naming_directory_loan = false;
            return Ok(false);
        }
        state.admission_loan = false;
        if let Some(naming_dir) = &mut state.naming_directory
            && !naming_dir.loan(handoff)?
        {
            state.naming_directory_loan = true;
            return Ok(false);
        }
        state.naming_directory_loan = false;
        match &mut state.directory {
            None => Ok(true),
            Some(original) => original.loan(handoff),
        }
    }
    pub fn return_source_retirement(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        let mut state = self.try_source_state()?;
        if state.admission_loan {
            state
                .admission
                .as_mut()
                .expect("original admission custody loan")
                .return_source_retirement(handoff)?;
            return Ok(false);
        }
        if state.naming_directory_loan {
            state
                .naming_directory
                .as_mut()
                .expect("original naming directory custody loan")
                .return_handoff(handoff)?;
            return Ok(false);
        }
        state
            .directory
            .as_mut()
            .expect("original selected directory loan")
            .return_handoff(handoff)
    }
}
