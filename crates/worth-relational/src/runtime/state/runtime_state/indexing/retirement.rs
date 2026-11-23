//! Select, loan, and return the exact serving index image during source close.
use super::RuntimeIndexingSubsystem;
use crate::durability::data::NativeDirectoryRetirementHandoff;
use crate::durability::medium::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial,
    RelationalCheckpointAcknowledgementDenial as Ack, RelationalDurableColdSource,
    RelationalDurableMemberLocator, RelationalDurableMemberRetirementObservation as Observation,
};
use crate::runtime::state::subsystems::history::history_source_retirement::{
    bind_source_retirement, RelationalSourceHistoryBinding,
};
use std::sync::Arc;

impl RuntimeIndexingSubsystem {
    pub(crate) fn carried_checkpoint_image(&self) -> Option<RelationalDurableMemberLocator> {
        self.cold
            .read()
            .as_ref()
            .map(|image| image.directory().manifest.derived_indexes)
    }

    pub(crate) fn source_retirement_history(
        &self,
    ) -> Result<Option<RelationalSourceHistoryBinding>, ()> {
        Ok(self
            .cold
            .try_read()
            .ok_or(())?
            .as_ref()
            .map(|image| bind_source_retirement(image.history())))
    }

    pub(crate) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Ack> {
        let image = {
            let mut cold = self.cold.try_write().ok_or(Ack::Busy)?;
            let Some(original) = cold.as_mut() else {
                return Ok(());
            };
            Arc::get_mut(original).ok_or(Ack::Busy)?;
            original.clone()
        };
        image.preflight_retirement(source)
    }

    pub(crate) fn loan_source_member(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        let cold = self.cold.try_read().ok_or(Denial::Busy)?;
        match cold.as_ref() {
            None => Ok(true),
            Some(image) => image.loan_source_member(handoff),
        }
    }

    pub(crate) fn return_source_member(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        self.cold
            .try_read()
            .ok_or(Denial::Busy)?
            .as_ref()
            .expect("original image loan")
            .return_source_member(handoff, source)
    }

    pub(crate) fn loan_source_directory(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        let cold = self.cold.try_read().ok_or(Ack::Busy)?;
        match cold.as_ref() {
            None => Ok(true),
            Some(image) => image.loan_source_directory(handoff),
        }
    }

    pub(crate) fn return_source_directory(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        self.cold
            .try_read()
            .ok_or(Ack::Busy)?
            .as_ref()
            .expect("original directory loan")
            .return_source_directory(handoff)
    }
}
