//! Sealed test doors over this module's private members.
use super::*;

impl CallerPassControl {
    #[doc(hidden)]
    pub fn run_interleaving_for_test(&self) {
        Self::run_interleaving(self)
    }
}
