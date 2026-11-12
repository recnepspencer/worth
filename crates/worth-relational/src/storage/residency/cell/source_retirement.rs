//! Actual region source slots use the existing cell and raw-cache gates.
use super::super::source_retirement::OriginalResidentSlot;
use super::*;
use crate::durability::medium::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial,
    RelationalDurableMemberRetirementObservation as Observation,
    RelationalDurableMemberRetirementRefusal as Refusal,
};

impl RelationalRegionResidency {
    fn try_source_state(&self) -> Result<std::sync::MutexGuard<'_, RegionState>, Denial> {
        match self.state.try_lock() {
            Ok(state) => Ok(state),
            Err(std::sync::TryLockError::Poisoned(error)) => Ok(error.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => Err(Denial::Busy),
        }
    }
    pub(in crate::storage::residency) fn source_retirement_symbols(
        &self,
        backing: bool,
    ) -> Option<crate::runtime::RuntimeSymbolTable> {
        if backing {
            self.backing.as_ref().map(|cold| cold.symbols.clone())
        } else {
            self.head_readmission
                .as_ref()
                .map(|owner| owner.symbols().clone())
        }
    }
    pub(in crate::storage::residency) fn preflight_source_member(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Denial> {
        let token = {
            let state = self.try_source_state()?;
            state
                .physical
                .as_ref()
                .map(|physical| physical.preflight_snapshot(source))
                .transpose()?
                .flatten()
        };
        if let Some(token) = token {
            token
                .preflight_retirement(source)
                .map_err(Denial::Refused)?;
        }
        Ok(())
    }
    pub(in crate::storage::residency) fn loan_source_member(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        match self.try_source_state()?.physical.as_mut() {
            Some(physical) => physical.loan(handoff),
            None => Ok(true),
        }
    }
    pub(in crate::storage::residency) fn return_source_member(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        self.try_source_state()?
            .physical
            .as_mut()
            .ok_or(Denial::Refused(Refusal::ForeignCustody))?
            .return_handoff(handoff, source)
    }
    pub(in crate::storage::residency) fn source_retirement_resident(
        &self,
        slot: OriginalResidentSlot,
    ) -> Result<Option<Arc<ResidentRegion>>, Denial> {
        match slot {
            OriginalResidentSlot::Current => Ok(match &self.try_source_state()?.contents {
                ResidencyState::Cold | ResidencyState::Damaged { .. } => None,
                ResidencyState::Head { region } | ResidencyState::Full(region) => {
                    Some(region.clone())
                }
            }),
            OriginalResidentSlot::RawHead | OriginalResidentSlot::RawFull => {
                let cache = if slot == OriginalResidentSlot::RawHead {
                    &self.raw_head
                } else {
                    &self.raw_full
                };
                let cached = match cache.try_lock() {
                    Ok(cached) => cached,
                    Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
                    Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
                };
                Ok(match &*cached {
                    Some(Ok(region)) => Some(region.clone()),
                    Some(Err(_)) | None => None,
                })
            }
            OriginalResidentSlot::Complete => Ok(None),
        }
    }
}
