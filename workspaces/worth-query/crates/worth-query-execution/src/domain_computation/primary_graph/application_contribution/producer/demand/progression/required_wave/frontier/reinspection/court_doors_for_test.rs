//! Sealed test doors over this module's private members.
use super::*;

impl<Ready> RequiredFrontier<Ready> {
    #[doc(hidden)]
    pub fn queued_member_for_test(&mut self, index: usize) -> Option<QueuedMember<'_, Ready>> {
        Self::queued_member(self, index)
    }
}
