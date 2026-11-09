//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductActivationRegistry {
    #[doc(hidden)]
    pub fn gate_for_test(
        &self,
        identity: &ProductBranchIdentity,
    ) -> Result<Arc<WorthQueryProductActivationGate>, WorthQueryProductActivationDenial> {
        Self::gate(self, identity)
    }
}

impl WorthQueryProductActivationRegistry {
    #[doc(hidden)]
    pub fn release_for_test(
        &self,
        identity: &ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
    ) -> bool {
        Self::release(self, identity, incarnation)
    }
}

impl WorthQueryProductActivationRegistry {
    #[doc(hidden)]
    pub fn reserve_for_test(
        &self,
    ) -> Result<WorthQueryProductActivationReservation, WorthQueryProductActivationDenial> {
        Self::reserve(self)
    }
}

impl WorthQueryProductActivationReservation {
    #[doc(hidden)]
    pub fn commit_for_test(self, branch: &ProductBranchObservation) {
        Self::commit(self, branch)
    }
}
