//! Prepared reads retain their genuine observation and original native payer.
use super::{DerivedRelationData, DerivedRelationHandle};
use crate::graph_member::NativeMemberKey;
use crate::validation::engine::InvariantSourceAffinity;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// The complete source interpretation admitted by a genuine structural view.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DerivedRelationReadShape {
    Adjacency { reverse: bool },
    Reachable { reverse: bool },
    Component,
}

/// A descriptive step retained by an actual admitted breadth-first traversal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedRelationWalkStep {
    pub member: NativeMemberKey,
    pub distance: usize,
}

/// A sealed lookup identity for one definition/root/posture/observation.
/// It cannot be deserialized or reconstructed from diagnostic coordinates.
#[derive(Clone, Debug)]
pub struct DerivedRelationReadIdentity {
    pub(super) source: InvariantSourceAffinity,
    pub(super) definition: DerivedRelationHandle,
    pub(super) root: NativeMemberKey,
    pub(super) shape: DerivedRelationReadShape,
}

impl DerivedRelationReadIdentity {
    pub const fn root(&self) -> &NativeMemberKey {
        &self.root
    }
    pub const fn shape(&self) -> DerivedRelationReadShape {
        self.shape
    }
}
impl PartialEq for DerivedRelationReadIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.definition == other.definition
            && self.root == other.root
            && self.shape == other.shape
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
        self.shape.hash(state);
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
    pub(super) steps: Vec<DerivedRelationWalkStep>,
    // Backing precedes its original meter so its reservation survives last drop.
    pub(super) _retention: crate::validation::CustomInvariantWorkMeter,
}

impl PreparedDerivedRelationRead {
    pub fn identity(&self) -> &DerivedRelationReadIdentity {
        &self.identity
    }
}
