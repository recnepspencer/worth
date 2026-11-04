//! Sealed test doors over this module's private members.
use super::*;

impl QueryRequestExecution<'_> {
    #[doc(hidden)]
    pub fn has_accounting_for_test(&self) -> bool {
        Self::has_accounting(self)
    }
}
