//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::semantic_source::SemanticSource;


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
