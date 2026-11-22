//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedScopeSelector {
    #[doc(hidden)]
    pub fn subject_for_test(&self) -> EntityId {
        Self::subject(self)
    }
}
