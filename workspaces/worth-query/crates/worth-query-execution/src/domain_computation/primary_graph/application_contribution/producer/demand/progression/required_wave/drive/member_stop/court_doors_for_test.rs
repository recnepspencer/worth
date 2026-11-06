//! Sealed test doors over this module's private members.
use super::*;

impl StopRecording {
    #[doc(hidden)]
    pub fn member_for_test() -> Self {
        Self::member()
    }
}
