//! Retire the original deferred image and traverse its detached history owner.
use super::*;
use worth_relational_storage::internal_port::storage::data::directory_source_edge::NativeDirectoryRetirementHandoff;
use worth_relational_storage::internal_port::durability::medium::member_custody::native_slot::{NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial};
use worth_relational_storage::internal_port::durability::medium::checkpoint_acknowledgement_denial::RelationalCheckpointAcknowledgementDenial as Ack;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementObservation as Observation;
impl DeferredIndexImage {
    pub fn directory(&self) -> Arc<RelationalCheckpointDirectory> {
        self.admitted
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .directory
            .snapshot()
            .expect("only a serving image exposes its source directory")
    }
    fn try_source_state(&self) -> Result<std::sync::MutexGuard<'_, DeferredIndexState>, Denial> {
        worth_execution::try_retirement_guard(self.admitted.try_lock(), Denial::Busy)
    }
}
impl DeferredIndexImage {
    pub fn preflight_retirement(
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

    pub fn loan_source_member(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        self.try_source_state()?.physical.loan(handoff)
    }

    pub fn return_source_member(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        self.try_source_state()?
            .physical
            .return_handoff(handoff, source)
    }

    pub fn loan_source_directory(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        self.try_source_state()
            .map_err(member_denial)?
            .directory
            .loan(handoff)
    }

    pub fn return_source_directory(
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
