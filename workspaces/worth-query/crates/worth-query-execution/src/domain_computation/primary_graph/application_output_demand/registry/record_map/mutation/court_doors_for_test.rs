//! Sealed test doors over this module's private members.
use super::*;

impl<'a> RecordMutation<'a> {
    #[doc(hidden)]
    pub fn finish_for_test(self) {
        Self::finish(self)
    }
}
