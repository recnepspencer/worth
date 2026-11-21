//! Sealed test doors over this module's private members.
use super::*;

impl FactMovement {
    #[doc(hidden)]
    pub fn movement_for_test(self) -> Movement {
        Self::movement(self)
    }
}
