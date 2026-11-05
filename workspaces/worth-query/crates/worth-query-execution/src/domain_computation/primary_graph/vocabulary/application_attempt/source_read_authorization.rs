//! Moved whole from `domain_computation::primary_graph::application_attempt::fact::source_read_authorization`: the items below and every impl they own.
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct SourceReadAuthorizationKey {
    pub source: [u8; 32],
    pub support: [u8; 32],
    pub principal: EntityId,
    pub scope: EntityId,
}
