//! Sealed test doors over this module's private members.
use super::*;

impl<Ready> MemberInput<Ready> {
    #[doc(hidden)]
    pub fn native_for_test(ready: Ready) -> Self {
        Self::native(ready)
    }
}
