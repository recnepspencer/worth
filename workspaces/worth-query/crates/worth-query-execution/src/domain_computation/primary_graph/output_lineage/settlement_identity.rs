use std::sync::Arc;

use super::{ProductCoordinate, SemanticSource};

/// The exact retained output selected by the lineage owner.
///
/// A fork may select an ancestor's settlement. Its identity therefore keeps the
/// recorded coordinate, rather than the reader's coordinate or an Arc address.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct RecordedSettlementIdentity {
    source: SemanticSource,
    coordinate: ProductCoordinate,
    slot: usize,
}

impl Ord for RecordedSettlementIdentity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.source
            .cmp(&other.source)
            .then_with(|| self.coordinate.occurrence.cmp(&other.coordinate.occurrence))
            .then_with(|| self.coordinate.generation.cmp(&other.coordinate.generation))
            .then_with(|| self.slot.cmp(&other.slot))
    }
}

impl PartialOrd for RecordedSettlementIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl RecordedSettlementIdentity {
    pub(in crate::domain_computation::primary_graph) fn source(&self) -> &SemanticSource {
        &self.source
    }
    pub(in crate::domain_computation::primary_graph) const fn address(
        &self,
    ) -> (
        worth_runtime_world::facade::ProductBranchIncarnation,
        u64,
        usize,
    ) {
        (
            self.coordinate.occurrence,
            self.coordinate.generation,
            self.slot,
        )
    }
    pub(super) const fn coordinate(&self) -> ProductCoordinate {
        self.coordinate
    }
    pub(super) const fn slot(&self) -> usize {
        self.slot
    }
    pub(super) fn retain(
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        slot: usize,
    ) -> Arc<Self> {
        Arc::new(Self {
            source: source.clone(),
            coordinate,
            slot,
        })
    }
}
