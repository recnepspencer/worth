//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn hold_required_successor_for_test(
        &self,
        successor: HeldRequiredSuccessor,
        key_of: impl FnOnce(&HeldRequiredSuccessor) -> Option<&WorthQueryOutputDemandKey>,
    ) -> Option<HeldRequiredSuccessor> {
        Self::hold_required_successor(self, successor, key_of)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn take_published_held_successor_for_test(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<Option<HeldRequiredSuccessor>, WorthQueryOutputDemandDenial> {
        Self::take_published_held_successor(self, key)
    }
}
