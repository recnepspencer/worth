//! Moved whole from `domain_computation::primary_graph::application_query::managed_index::source_rows`: the items below and every impl they own.
use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity;

/// A descriptive source slot issued by the selected source owner. It is not an entity handle.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryManagedIndexSourceSlot {
    pub(in crate::domain_computation::primary_graph) owner: WorthQueryGraphWorkSessionIdentity,
    pub(in crate::domain_computation::primary_graph) ordinal: usize,
    pub(in crate::domain_computation::primary_graph) entity:
        Option<worth_relational::facade::identity::EntityId>,
}
