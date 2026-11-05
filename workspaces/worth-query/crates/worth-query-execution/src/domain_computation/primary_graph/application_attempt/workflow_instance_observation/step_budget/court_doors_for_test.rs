//! Sealed test doors over this module's private members.
use super::*;

impl WorkflowStepAllowance {
    #[doc(hidden)]
    pub fn for_test_for_test() -> Self {
        Self::for_test()
    }
}
