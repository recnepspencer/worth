//! Sealed test doors over this module's private members.
use super::*;

impl RecordedSettlementIdentity {
    #[doc(hidden)]
    pub fn address_for_test(
        &self,
    ) -> (
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

impl RecordedSettlementIdentity {
    #[doc(hidden)]
    pub fn retain_for_test(
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        slot: usize,
    ) -> Arc<Self> {
        Self::retain(source, coordinate, slot)
    }
}

impl RecordedSettlementIdentity {
    #[doc(hidden)]
    pub fn coordinate_for_test(&self) -> ProductCoordinate {
        Self::coordinate(self)
    }
}
