//! Sealed test doors over this module's private members.
use super::*;

impl NativeMarkingReport {
    #[doc(hidden)]
    pub fn precision_for_test(&self) -> &WorthQueryNativeMarkingPrecision {
        &self.precision
    }
}
