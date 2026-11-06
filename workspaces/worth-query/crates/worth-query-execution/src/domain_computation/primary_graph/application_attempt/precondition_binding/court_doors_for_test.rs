//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryBoundMutationPreconditions {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> Option<&[u8; 32]> {
        Self::identity(self)
    }
}
