//! Sealed test doors over this module's private members.
use super::*;

impl PanicDuringPinGuardForTest {
    #[doc(hidden)]
    pub fn install_on_this_thread_for_test() -> Self {
        Self::install_on_this_thread()
    }
}
