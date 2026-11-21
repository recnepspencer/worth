//! Sealed test doors over this module's private members.
use super::*;

impl RawNonLiveKernelOutcome {
    #[doc(hidden)]
    pub fn raw_for_test(&self) -> &RawOneShotRows {
        &self.raw
    }
}

impl RawOneShotRows {
    #[doc(hidden)]
    pub fn rows_for_test(&self) -> &WorthQueryApplicationDisclosedProjectionTree {
        &self.rows
    }
}
