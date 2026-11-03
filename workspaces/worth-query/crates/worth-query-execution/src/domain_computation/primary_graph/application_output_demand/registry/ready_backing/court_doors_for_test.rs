//! Sealed test doors over this module's private members.
use super::*;

impl PreparedReadyBacking {
    #[doc(hidden)]
    pub fn new_for_test() -> Self {
        Self::new()
    }
}
