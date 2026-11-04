//! Sealed test doors over this module's private members.
use super::*;

impl QueryReadBinding {
    #[doc(hidden)]
    pub fn dirty_for_test(&self) -> &Option<usize> {
        &self.dirty
    }
}

impl QueryReadBinding {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &Weak<QueryReadIdentity> {
        &self.identity
    }
}

impl QueryReadBinding {
    #[doc(hidden)]
    pub fn projection_for_test(&self) -> &QueryReadBacking {
        &self.projection
    }
}
