//! Sealed test doors over this module's private members.
use super::*;

impl StructuralWalkKey {
    #[doc(hidden)]
    pub fn roots_for_test(&self) -> &[EntityId] {
        Self::roots(self)
    }
}
