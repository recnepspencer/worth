//! Moved whole from `domain_computation::primary_graph::application_query::observed_source::source_identity`: the items below and every impl they own.

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryObservedSourceOccurrence {
    Relational,
    Product(worth_runtime_world::facade::ProductBranchIncarnation),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryObservedSourceCoordinate {
    pub(in crate::domain_computation::primary_graph) query: [u8; 32],
    pub(in crate::domain_computation::primary_graph) parameters: [u8; 32],
    pub(in crate::domain_computation::primary_graph) occurrence: WorthQueryObservedSourceOccurrence,
    pub(in crate::domain_computation::primary_graph) root:
        worth_relational::facade::identity::EntityId,
}
