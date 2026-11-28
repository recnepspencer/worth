//! Retire only this envelope cell's original physical source edges.
use super::*;
use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
use crate::storage::data::directory_source_edge::NativeDirectoryRetirementHandoff;
impl RelationalCommitEnvelopeResidency {
    fn try_source_state(&self) -> Result<std::sync::MutexGuard<'_, EnvelopeState>, Ack> {
        worth_execution::try_retirement_guard(self.state.try_lock(), Ack::Busy)
    }
    pub fn preflight_source_retirement(
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
    pub fn loan_source_retirement(
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
    pub fn return_source_retirement(
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
