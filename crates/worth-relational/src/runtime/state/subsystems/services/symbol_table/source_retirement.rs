//! Source retirement follows this actual table's original cold-directory slot.
use super::*;
use crate::durability::medium::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial, RelationalDurableColdSource,
    RelationalDurableMemberRetirementObservation as Observation,
    RelationalDurableMemberRetirementRefusal as Refusal,
};

impl RuntimeSymbolTable {
    pub(crate) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Denial> {
        let target = {
            let mut slot = match self.cold.try_write() {
                Ok(slot) => slot,
                Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
                Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
            };
            match slot.as_mut() {
                None => None,
                Some(original) => {
                    // A live exported source owns this residency Arc. Table
                    // metadata aliases share the cold slot instead and remain
                    // lawful. No reference count is used as an authority probe.
                    Arc::get_mut(original).ok_or(Denial::Busy)?;
                    Some(original.clone())
                }
            }
        };
        if let Some(target) = target {
            target.preflight_source_retirement(source)?;
        }
        Ok(())
    }
    pub(crate) fn loan_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        let slot = match self.cold.try_read() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
        };
        match slot.as_ref() {
            Some(original) => original.loan_source_retirement(handoff),
            None => Ok(true),
        }
    }
    pub(crate) fn return_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        let slot = match self.cold.try_read() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
        };
        slot.as_ref()
            .ok_or(Denial::Refused(Refusal::ForeignCustody))?
            .return_source_retirement(handoff, source)
    }
}
