//! The original recovery checkpoint owns its directory retirement through reclaim.
use super::*;
use crate::durability::data::NativeDirectoryRetirementHandoff;
use crate::runtime::{RelationalRuntimeCloseDenial, RelationalSourceCloseDenial as Stop};

fn shared_checkpoint() -> Stop {
    Stop::Native(RelationalRuntimeCloseDenial::ReadCustodyRetained)
}

impl DurabilitySubsystem {
    pub(crate) fn preflight_checkpoint_directory_retirement(
        &self,
        index: usize,
        canonical: bool,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<bool, Stop> {
        let original = {
            let state = self.state.try_read().ok_or(Stop::Busy)?;
            let Some(checkpoint) = state.checkpoints.get(index) else {
                return Ok(false);
            };
            let directory = if canonical {
                &checkpoint.canonical_directory
            } else {
                &checkpoint.directory
            };
            if directory.is_some() && Arc::strong_count(checkpoint) != 1 {
                return Err(shared_checkpoint());
            }
            directory.as_ref().and_then(|edge| edge.snapshot())
        };
        // Provider preflight runs outside the Native metadata gate.
        if let Some(original) = original {
            original
                .preflight_retirement(source)
                .map_err(Stop::Checkpoint)?;
        }
        Ok(true)
    }

    pub(crate) fn loan_checkpoint_directory_retirement(
        &self,
        index: usize,
        canonical: bool,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<Option<bool>, Stop> {
        let mut state = self.state.try_write().ok_or(Stop::Busy)?;
        let Some(checkpoint) = state.checkpoints.get_mut(index) else {
            return Ok(None);
        };
        if if canonical {
            checkpoint.canonical_directory.is_none()
        } else {
            checkpoint.directory.is_none()
        } {
            return Ok(Some(true));
        }
        let checkpoint = Arc::get_mut(checkpoint).ok_or_else(shared_checkpoint)?;
        let directory = if canonical {
            &mut checkpoint.canonical_directory
        } else {
            &mut checkpoint.directory
        };
        directory
            .as_mut()
            .expect("same original directory edge")
            .loan(handoff)
            .map(Some)
            .map_err(Stop::Checkpoint)
    }

    pub(crate) fn return_checkpoint_directory_retirement(
        &self,
        index: usize,
        canonical: bool,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Stop> {
        let mut state = self.state.try_write().ok_or(Stop::Busy)?;
        let checkpoint = state
            .checkpoints
            .get_mut(index)
            .expect("same checkpoint owns its loan");
        let checkpoint = Arc::get_mut(checkpoint).ok_or_else(shared_checkpoint)?;
        let directory = if canonical {
            &mut checkpoint.canonical_directory
        } else {
            &mut checkpoint.directory
        };
        directory
            .as_mut()
            .expect("same original directory loan")
            .return_handoff(handoff)
            .map_err(Stop::Checkpoint)
    }
}
