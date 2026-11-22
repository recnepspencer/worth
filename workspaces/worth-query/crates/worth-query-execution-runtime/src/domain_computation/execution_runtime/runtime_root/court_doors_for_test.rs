//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryExecutionRuntime {
    #[doc(hidden)]
    pub fn retain_current_generation_for_test(&self) -> Arc<AtomicU64> {
        Self::retain_current_generation(self)
    }
}
