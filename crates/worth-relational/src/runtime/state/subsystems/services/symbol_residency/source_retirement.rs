//! The original dictionary page lock also owns its physical custody slot.
use super::*;
use crate::durability::medium::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial,
    RelationalDurableMemberRetirementObservation as Observation,
};

#[derive(Debug)]
pub(super) struct SymbolPageState {
    pub(super) data: BTreeMap<
        RelationalDurableMemberLocator,
        Result<SymbolPageShape, RelationalStorageUnavailable>,
    >,
    pub(super) physical: crate::durability::medium::NativeSourceMemberState,
}
impl std::ops::Deref for SymbolPageState {
    type Target = BTreeMap<
        RelationalDurableMemberLocator,
        Result<SymbolPageShape, RelationalStorageUnavailable>,
    >;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl std::ops::DerefMut for SymbolPageState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl RuntimeSymbolResidency {
    fn try_source_state(&self) -> Result<std::sync::MutexGuard<'_, SymbolPageState>, Denial> {
        match self.pages.try_lock() {
            Ok(state) => Ok(state),
            Err(std::sync::TryLockError::Poisoned(error)) => Ok(error.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => Err(Denial::Busy),
        }
    }

    pub(in crate::runtime) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Denial> {
        let snapshot = {
            let pages = self.try_source_state()?;
            pages.physical.preflight_snapshot(source)?
        };
        if let Some(token) = snapshot {
            token
                .preflight_retirement(source)
                .map_err(Denial::Refused)?;
        }
        Ok(())
    }
    pub(in crate::runtime) fn loan_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        let mut pages = self.try_source_state()?;
        pages.physical.loan(handoff)
    }
    pub(in crate::runtime) fn return_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        let mut pages = self.try_source_state()?;
        pages.physical.return_handoff(handoff, source)
    }
}
