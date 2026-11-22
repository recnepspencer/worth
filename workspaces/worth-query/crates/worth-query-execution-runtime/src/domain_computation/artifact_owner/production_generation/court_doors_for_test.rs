//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryArtifactProductionGeneration {
    #[doc(hidden)]
    pub fn initial_for_test() -> Self {
        Self::initial()
    }
}
