//! Immutable subject of the physically acknowledged original open.
use super::WorldOpenedPublicationAttempt;
use worth_foundational::facade::{CanonicalDigestId, CompositeAttemptAssociation};

impl WorldOpenedPublicationAttempt {
    /// The selected program recorded in this exact acknowledged open.
    pub fn program_revision(&self) -> CanonicalDigestId {
        CanonicalDigestId::new(self.original.open_record().program_revision)
    }
    /// Durable coordinates from the same record, never inferred from a live head.
    pub fn association(&self) -> &CompositeAttemptAssociation {
        &self.original.open_record().association
    }
}
