//! A bounded, portable description of source-native authorization reads.
//!
//! This is descriptive evidence, never a transferable authorization permit.

use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, KindId, RelationId, VersionId};
use crate::runtime::RelationalRuntime;

use super::{
    RelationalAuthorizationObservationEvidence, RelationalAuthorizationTraversalDirection,
};

mod capture;
mod wire;

pub const MAXIMUM_AUTHORIZATION_DEPENDENCIES: usize = 4_096;
pub const MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES: usize = 65_536;
const DURABLE_DEPENDENCY_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalAuthorizationDependencyDenial {
    ForeignRuntime,
    SnapshotUnavailable,
    InexactSnapshot,
    IncompleteObservation,
    DependencyUnavailable,
    UnsupportedFieldLocator,
    DependencyBudgetExceeded,
    ByteBudgetExceeded,
    MalformedWire,
    UnsupportedVersion,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalAuthorizationDurableDependencies {
    version: u8,
    principal: EntityDependency,
    scope: EntityDependency,
    paths: Vec<PathDependencies>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PathDependencies {
    matched: bool,
    witness: Option<Vec<EntityId>>,
    entities: Vec<EntityDependency>,
    relations: Vec<RelationDependency>,
    adjacencies: Vec<AdjacencyDependency>,
    fields: Vec<FieldDependency>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EntityDependency {
    entity: EntityId,
    kind: KindId,
    created_at: VersionId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RelationDependency {
    relation: RelationId,
    kind: KindId,
    created_at: VersionId,
    structural_revision: VersionId,
    source: EntityId,
    target: EntityId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdjacencyDependency {
    entity: EntityId,
    relation_kind: KindId,
    direction: RelationalAuthorizationTraversalDirection,
    revision: Option<VersionId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldDependency {
    entity: EntityId,
    locator_authority: FieldLocatorAuthority,
    aspect: String,
    field: String,
    revision: crate::storage::data::RelationalFieldRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum FieldLocatorAuthority {
    Authoritative,
    Derived,
    Projected,
    SupportOnly,
    Planned,
    ReceiptBearing,
}

impl RelationalAuthorizationDurableDependencies {
    /// Equality is meaningful only against a fresh Relational-owned capture.
    pub fn matches(&self, fresh: &Self) -> bool {
        self == fresh && self.version == DURABLE_DEPENDENCY_VERSION
    }
}

impl RelationalRuntime {
    /// Capture exact source revisions from the evidence's own immutable root.
    /// Cost is O(tracked reads); no graph reconstruction is permitted here.
    pub fn capture_authorization_durable_dependencies(
        &self,
        evidence: &RelationalAuthorizationObservationEvidence,
    ) -> Result<RelationalAuthorizationDurableDependencies, RelationalAuthorizationDependencyDenial>
    {
        capture::capture(self, evidence)
    }
}
