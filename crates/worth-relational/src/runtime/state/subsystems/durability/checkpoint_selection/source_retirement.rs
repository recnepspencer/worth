//! Nonwaiting access to the original selected directory slot.
use super::*;
use crate::durability::data::NativeDirectoryRetirementHandoff;
use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
impl RelationalCheckpointManifestSelection {
    fn try_source_state(&self) -> Result<std::sync::MutexGuard<'_, CheckpointSelectionState>, Ack> {
        match self.selected.try_lock() {
            Ok(state) => Ok(state),
            Err(std::sync::TryLockError::Poisoned(error)) => Ok(error.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => Err(Ack::Busy),
        }
    }
    pub(crate) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Ack> {
        let (original, admission) = {
            let state = self.try_source_state()?;
            (
                state
                    .directory
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
        Ok(())
    }
    pub(crate) fn loan_source_retirement(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        let mut state = self.try_source_state()?;
        if let Some(admission) = &mut state.admission {
            if !admission.loan_source_retirement(handoff)? {
                state.admission_loan = true;
                return Ok(false);
            }
        }
        state.admission_loan = false;
        match &mut state.directory {
            None => Ok(true),
            Some(original) => original.loan(handoff),
        }
    }
    pub(crate) fn return_source_retirement(
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
        state
            .directory
            .as_mut()
            .expect("original selected directory loan")
            .return_handoff(handoff)
    }
}
