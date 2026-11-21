//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRuntimeClock {
    #[doc(hidden)]
    pub fn from_source_for_test(source: impl WorthQueryRuntimeTimeSource) -> Self {
        Self::from_source(source)
    }
}
