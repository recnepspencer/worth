//! Owned relation meaning at schema erasure and portable-record boundaries.
use super::{
    ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints, ApplicationRelationIntegrity,
    ApplicationRelationUniquenessScope,
};

mod minimum;
pub use minimum::ApplicationRelationMinimumReachabilityRecord;

/// Owns the law data after typed const authoring or untrusted archive decoding.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRelationIntegrityRecord {
    pub endpoints: ApplicationRelationEndpoints,
    pub cardinality: ApplicationRelationCardinality,
    pub deletion: ApplicationRelationDeletionPolicy,
    pub acyclic: bool,
    pub symmetric: bool,
    pub orbit: bool,
    pub unique: Option<ApplicationRelationUniquenessScope>,
    pub same_partition_endpoints: bool,
    pub reaches_at_least: Option<ApplicationRelationMinimumReachabilityRecord>,
}

impl ApplicationRelationIntegrityRecord {
    pub const fn cross_context_policy(&self) -> ApplicationRelationCrossContextPolicy {
        self.endpoints.cross_context_policy
    }
}

impl From<ApplicationRelationIntegrity> for ApplicationRelationIntegrityRecord {
    fn from(authored: ApplicationRelationIntegrity) -> Self {
        Self {
            endpoints: authored.endpoints,
            cardinality: authored.cardinality,
            deletion: authored.deletion,
            acyclic: authored.acyclic,
            symmetric: authored.symmetric,
            orbit: authored.orbit,
            unique: authored.unique,
            same_partition_endpoints: authored.same_partition_endpoints,
            reaches_at_least: authored.reaches_at_least.map(Into::into),
        }
    }
}
