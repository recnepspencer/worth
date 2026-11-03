//! Sealed test doors over this module's private members.
use super::*;

impl RecordedSettlementIdentity {
    #[doc(hidden)]
    pub fn address_for_test(
        &self) -> (
        worth_runtime_world::facade::ProductBranchIncarnation,
        u64,
        usize,
    ) {
        Self::address(self)
    }
}

impl RecordedSettlementIdentity {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &SemanticSource {
        Self::source(self)
    }
}
