//! The original component intent determines each Query-owned World phase.
use worth_runtime_world::facade::{
    PlannedProductReferenceSuccessor, WorldPreparedPublicationWithSignal,
    WorldPreparedPublicationWithoutSignal, WorldQueryNativePreparation,
    WorldQuerySignalPreparation,
};
pub(in crate::domain_computation::primary_graph::provider) enum QueryBoundWorldPreparation {
    Relational(WorldQueryNativePreparation),
    WithSignal(WorldQuerySignalPreparation),
}
pub(in crate::domain_computation::primary_graph::provider) enum QueryPreparedWorldPublication {
    Relational(WorldPreparedPublicationWithoutSignal),
    WithSignal(WorldPreparedPublicationWithSignal),
}
impl QueryPreparedWorldPublication {
    pub(super) fn planned_successor(&self) -> &PlannedProductReferenceSuccessor {
        match self {
            Self::Relational(prepared) => prepared.planned_successor(),
            Self::WithSignal(prepared) => prepared.planned_successor(),
        }
    }
}
