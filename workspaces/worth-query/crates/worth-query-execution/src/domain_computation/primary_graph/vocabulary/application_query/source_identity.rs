//! Moved whole from `domain_computation::primary_graph::application_query::observed_source::source_identity`: the items below and every impl they own.

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryObservedSourceOccurrence {
    Relational,
    Product(worth_runtime_world::facade::ProductBranchIncarnation),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryObservedSourceCoordinate {
    query: [u8; 32],
    parameters: [u8; 32],
    occurrence: WorthQueryObservedSourceOccurrence,
    root: worth_relational::facade::identity::EntityId,
}
