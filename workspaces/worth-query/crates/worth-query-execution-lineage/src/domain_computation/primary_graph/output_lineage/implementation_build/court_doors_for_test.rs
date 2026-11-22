//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn bind_implementation_build_for_test(&mut self, build: ApplicationBuildIdentity) {
        Self::bind_implementation_build(self, build)
    }
}
