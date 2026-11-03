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

impl ApplicationFactTable {
    #[doc(hidden)]
    pub fn get_for_test(&self, key: &str) -> Option<Decision> {
        Self::get(self, key)
    }
}

impl ApplicationFactTable {
    #[doc(hidden)]
    pub fn requests_for_test(
        &self) -> impl Iterator<Item = crate::domain_computation::WorthQueryDecisionFactRequest> + '_ {
        Self::requests(self)
    }
}

impl ApplicationFactTable {
    #[doc(hidden)]
    pub fn values_for_test(&self) -> impl Iterator<Item = Decision> + '_ {
        Self::values(self)
    }
}
