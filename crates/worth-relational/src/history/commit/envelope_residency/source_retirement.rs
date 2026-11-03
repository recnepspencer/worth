//! Retire only this envelope cell's original physical source edges.
use super::*;
use crate::durability::data::NativeDirectoryRetirementHandoff;
use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
impl RelationalCommitEnvelopeResidency {
    fn try_source_state(&self) -> Result<std::sync::MutexGuard<'_, EnvelopeState>, Ack> {
        match self.state.try_lock() {
            Ok(state) => Ok(state),
            Err(std::sync::TryLockError::Poisoned(error)) => Ok(error.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => Err(Ack::Busy),
        }
    }
    pub(crate) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Ack> {
        let directory = {
            let state = self.try_source_state()?;
            match &state.backing {
                EnvelopeBacking::Memory { .. } => None,
                EnvelopeBacking::Journal {
                    source: original, ..
                } => {
                    original.preflight(source).map_err(Ack::Retirement)?;
                    None
                }
                EnvelopeBacking::SelectedCheckpoint { directory, .. } => directory.snapshot(),
            }
        };
        if let Some(directory) = directory {
            directory.preflight_retirement(source)?;
        }
        Ok(())
    }
    pub(crate) fn loan_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        let mut state = self.try_source_state()?;
        match &mut state.backing {
            EnvelopeBacking::Memory { .. } => Ok(true),
            EnvelopeBacking::Journal {
                source: original, ..
            } => {
                original.retire(source).map_err(Ack::Retirement)?;
                Ok(true)
            }
            EnvelopeBacking::SelectedCheckpoint { directory, .. } => directory.loan(handoff),
        }
    }
    pub(crate) fn return_source_retirement(
        &self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        let mut state = self.try_source_state()?;
        match &mut state.backing {
            EnvelopeBacking::SelectedCheckpoint { directory, .. } => {
                directory.return_handoff(handoff)
            }
            EnvelopeBacking::Memory { .. } | EnvelopeBacking::Journal { .. } => {
                unreachable!("only the original selected edge loans a task")
            }
        }
    }
}
