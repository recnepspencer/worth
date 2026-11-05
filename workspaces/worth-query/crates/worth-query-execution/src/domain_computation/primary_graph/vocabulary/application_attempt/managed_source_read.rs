//! Moved whole from `domain_computation::primary_graph::application_attempt::fact::managed_source_read`: the items below and every impl they own.
use worth_relational::facade::identity::EntityId;

/// Descriptive locator within the already checked schema/runtime/branch affinity.
/// The retained Native observation, not this key, supplies Source currentness.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct ManagedSourceReadKey {
    pub obligation: [u8; 32],
    pub authorization: [u8; 32],
    pub scope: EntityId,
    pub influence: Option<ManagedInfluenceOrigin>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct ManagedInfluenceOrigin {
    pub parameters: [u8; 32],
    pub audience: [u8; 32],
    pub content: worth_foundational::facade::ProjectedCandidateContentV1,
}
