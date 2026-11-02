//! The original source edge and its issued physical member registration.
#[cfg(test)]
mod tests;
use super::native_slot::{
    NativeMemberRetirementHandoff, NativeMemberRetirementSlot, NativeMemberSlotDenial as Denial,
};
use super::*;

#[derive(Debug)]
pub(crate) struct NativeSourceMemberState {
    source: Option<Arc<dyn RelationalDurableColdSource>>,
    custody: NativeMemberRetirementSlot,
}
impl NativeSourceMemberState {
    pub(crate) fn new(
        source: Arc<dyn RelationalDurableColdSource>,
        token: RelationalDurableMemberCustody,
    ) -> Self {
        Self {
            source: Some(source),
            custody: NativeMemberRetirementSlot::Original(token),
        }
    }
    pub(crate) fn source(&self) -> Option<&Arc<dyn RelationalDurableColdSource>> {
        self.source.as_ref()
    }
    pub(crate) fn retain_serving(&self) -> Self {
        Self {
            source: self.source.clone(),
            custody: self.custody.retain_serving(),
        }
    }
    /// Clone only this original token share under the containing lock; invoke
    /// its non-consuming provider preflight after releasing that lock.
    pub(crate) fn preflight_snapshot(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Option<RelationalDurableMemberCustody>, Denial> {
        if let Some(original) = &self.source {
            if !std::ptr::addr_eq(original.as_ref(), source) {
                return Err(Denial::Refused(
                    RelationalDurableMemberRetirementRefusal::ForeignSource,
                ));
            }
        }
        match &self.custody {
            NativeMemberRetirementSlot::Original(token) => Ok(Some(token.clone())),
            NativeMemberRetirementSlot::Retired { .. } => Ok(None),
            NativeMemberRetirementSlot::Retiring(_) | NativeMemberRetirementSlot::Loaned => {
                Err(Denial::Busy)
            }
        }
    }
    pub(crate) fn loan(
        &mut self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        assert!(handoff.is_none(), "a returning task is never replaced");
        *handoff = self.custody.loan()?;
        Ok(handoff.is_none())
    }
    pub(crate) fn return_handoff(
        &mut self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<RelationalDurableMemberRetirementObservation, Denial> {
        let original = self
            .source
            .as_ref()
            .expect("loaned member retains its original source");
        if !std::ptr::addr_eq(original.as_ref(), source) {
            return Err(Denial::Refused(
                RelationalDurableMemberRetirementRefusal::ForeignSource,
            ));
        }
        let observation = self
            .custody
            .return_handoff(handoff.take().expect("same original handoff"));
        if matches!(
            observation,
            RelationalDurableMemberRetirementObservation::Complete
        ) {
            // Matching borrowed actual source guarantees this is a local edge.
            drop(self.source.take());
        }
        Ok(observation)
    }
}
