//! Retire the original deferred image and traverse its detached history owner.
use super::*;
use crate::durability::data::NativeDirectoryRetirementHandoff;
use crate::durability::medium::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial,
    RelationalCheckpointAcknowledgementDenial as Ack, RelationalDurableColdSource,
    RelationalDurableMemberRetirementObservation as Observation,
};
use crate::runtime::state::subsystems::RelationalSourceHistoryBinding;
impl DeferredIndexImage {
    pub(super) fn directory(&self) -> Arc<RelationalCheckpointDirectory> {
        self.admitted
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .directory
            .snapshot()
            .expect("only a serving image exposes its source directory")
    }
    fn try_source_state(&self) -> Result<std::sync::MutexGuard<'_, DeferredIndexState>, Denial> {
        match self.admitted.try_lock() {
            Ok(state) => Ok(state),
            Err(std::sync::TryLockError::Poisoned(error)) => Ok(error.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => Err(Denial::Busy),
        }
    }
}
impl IndexingSubsystem {
    pub(crate) fn source_retirement_history(
        &self,
    ) -> Result<Option<RelationalSourceHistoryBinding>, ()> {
        Ok(self
            .state
            .try_read()
            .ok_or(())?
            .cold_checkpoint
            .as_ref()
            .map(|image| image.history.source_retirement_binding()))
    }
    pub(crate) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Ack> {
        let image = {
            let mut state = self.state.try_write().ok_or(Ack::Busy)?;
            match state.cold_checkpoint.as_mut() {
                None => return Ok(()),
                Some(original) => {
                    Arc::get_mut(original).ok_or(Ack::Busy)?;
                    original.clone()
                }
            }
        };
        let (directory, token) = {
            let state = image.try_source_state().map_err(member_denial)?;
            (
                state.directory.snapshot(),
                state
                    .physical
                    .preflight_snapshot(source)
                    .map_err(member_denial)?,
            )
        };
        if let Some(token) = token {
            token
                .preflight_retirement(source)
                .map_err(Ack::Retirement)?;
        }
        if let Some(directory) = directory {
            directory.preflight_retirement(source)?;
        }
        Ok(())
    }
    pub(crate) fn loan_source_member(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        let state = self.state.try_read().ok_or(Denial::Busy)?;
        match &state.cold_checkpoint {
            None => Ok(true),
            Some(image) => image.try_source_state()?.physical.loan(handoff),
        }
    }
    pub(crate) fn return_source_member(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        let state = self.state.try_read().ok_or(Denial::Busy)?;
        state
            .cold_checkpoint
            .as_ref()
            .expect("original image loan")
            .try_source_state()?
            .physical
            .return_handoff(handoff, source)
    }
    pub(crate) fn loan_source_directory(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        let state = self.state.try_read().ok_or(Ack::Busy)?;
        match &state.cold_checkpoint {
            None => Ok(true),
            Some(image) => image
                .try_source_state()
                .map_err(member_denial)?
                .directory
                .loan(handoff),
        }
    }
    pub(crate) fn return_source_directory(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        let state = self.state.try_read().ok_or(Ack::Busy)?;
        state
            .cold_checkpoint
            .as_ref()
            .expect("original directory loan")
            .try_source_state()
            .map_err(member_denial)?
            .directory
            .return_handoff(handoff)
    }
}
fn member_denial(cause: Denial) -> Ack {
    match cause {
        Denial::Busy => Ack::Busy,
        Denial::Refused(cause) => Ack::Retirement(cause),
    }
}
