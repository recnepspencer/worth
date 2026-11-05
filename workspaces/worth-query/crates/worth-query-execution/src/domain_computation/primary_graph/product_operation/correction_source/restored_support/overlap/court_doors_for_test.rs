//! Sealed test doors over this module's private members.
use super::*;

impl Selection {
    #[doc(hidden)]
    pub fn keys_for_test(&self) -> &ExecutionArray<Key> {
        &self.keys
    }
}
