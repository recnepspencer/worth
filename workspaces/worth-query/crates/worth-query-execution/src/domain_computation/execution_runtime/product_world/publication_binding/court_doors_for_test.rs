//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductPublicationBinding {
    #[doc(hidden)]
    pub fn observation_for_test(&self) -> &ProductBranchObservation {
        Self::observation(self)
    }
}
