//! Prepared reads retain their genuine observation and original native payer.
use super::affinity::DerivedSourceAffinity;
use super::{DerivedRelationData, DerivedRelationHandle};
use crate::identity::data::EntityId;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A sealed lookup identity for one definition/root/posture/observation.
/// It cannot be deserialized or reconstructed from diagnostic coordinates.
#[derive(Clone, Debug)]
pub struct DerivedRelationReadIdentity {
    pub(super) source: DerivedSourceAffinity,
    pub(super) definition: DerivedRelationHandle,
    pub(super) root: EntityId,
}

impl DerivedRelationReadIdentity {
    pub const fn root(&self) -> EntityId {
        self.root
    }
}
impl PartialEq for DerivedRelationReadIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.definition == other.definition
            && self.root == other.root
    }
}
impl Eq for DerivedRelationReadIdentity {}
impl Hash for DerivedRelationReadIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Membership only: no ordering, serialized key or graph identity follows
        // from these opaque owners. Full equality also compares the proposal.
        self.source.observation.hash(state);
        Arc::as_ptr(&self.definition.entry).hash(state);
        self.source.committed.hash(state);
        self.source.version.hash(state);
        self.root.hash(state);
    }
}

/// Complete data admitted during planning. Only the same immutable view may
/// consume it after freeze; the data by itself grants no graph authority.
#[derive(Clone)]
pub struct PreparedDerivedRelationRead {
    pub(super) identity: DerivedRelationReadIdentity,
    pub(super) body: Arc<PreparedBody>,
}

pub(super) struct PreparedBody {
    pub(super) data: DerivedRelationData,
    // Backing precedes its original meter so its reservation survives last drop.
    pub(super) _retention: crate::validation::CustomInvariantWorkMeter,
}

impl PreparedDerivedRelationRead {
    pub fn identity(&self) -> &DerivedRelationReadIdentity {
        &self.identity
    }
}
