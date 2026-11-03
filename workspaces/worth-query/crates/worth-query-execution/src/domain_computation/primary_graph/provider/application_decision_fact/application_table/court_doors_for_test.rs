//! Sealed test doors over this module's private members.
use super::*;

impl ApplicationFactTable {
    #[doc(hidden)]
    pub fn closed_prefix_for_test(
        &self) -> Option<(&Arc<CompletedFactProjection>, &Arc<CompletedOriginalRead>)> {
        Self::closed_prefix(self)
    }
}

impl ApplicationFactTable {
    #[doc(hidden)]
    pub fn full_capture_bytes_for_test(&self) -> Option<u64> {
        Self::full_capture_bytes(self)
    }
}

impl ApplicationFactTable {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}
