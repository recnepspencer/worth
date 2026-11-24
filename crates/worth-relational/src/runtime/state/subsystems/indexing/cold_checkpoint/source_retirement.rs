//! Retire the original deferred image and traverse its detached history owner.
use super::*;
use crate::durability::data::NativeDirectoryRetirementHandoff;
use crate::durability::medium::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial,
    RelationalCheckpointAcknowledgementDenial as Ack, RelationalDurableColdSource,
    RelationalDurableMemberRetirementObservation as Observation,
};
impl DeferredIndexImage {
    pub(crate) fn directory(&self) -> Arc<RelationalCheckpointDirectory> {
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
impl DeferredIndexImage {
    pub(crate) fn preflight_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Ack> {
        let (directory, token) = {
            let state = self.try_source_state().map_err(member_denial)?;
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
        self.try_source_state()?.physical.loan(handoff)
    }

    pub(crate) fn return_source_member(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        self.try_source_state()?
            .physical
            .return_handoff(handoff, source)
    }

    pub(crate) fn loan_source_directory(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        self.try_source_state()
            .map_err(member_denial)?
            .directory
            .loan(handoff)
    }

    pub(crate) fn return_source_directory(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        self.try_source_state()
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
