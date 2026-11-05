//! Sealed test doors over this module's private members.
use super::*;

impl ComputedMember {
    #[doc(hidden)]
    pub fn into_slots_for_test(self) -> Vec<ComputedSlot> {
        Self::into_slots(self)
    }
}
