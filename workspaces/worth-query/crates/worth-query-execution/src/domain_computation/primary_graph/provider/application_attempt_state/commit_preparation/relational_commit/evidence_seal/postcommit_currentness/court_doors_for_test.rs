//! Sealed test doors over this module's private members.
use super::*;

impl RebasedSourceFacts {
    #[doc(hidden)]
    pub fn retain_exact_for_test(&self) -> Option<FactPayload> {
        Self::retain_exact(self)
    }
}

impl OwnEffectOnReads {
    #[doc(hidden)]
    pub fn new_for_test(field0: OwnEffect) -> Self {
        Self(field0)
    }
}
