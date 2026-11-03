//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn complete_prepared_source_for_test(
        &self, receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt) {
        Self::complete_prepared_source(self, receipt)
    }
}
