//! A bounded, portable description of source-native authorization reads.
//!
//! This is descriptive evidence, never a transferable authorization permit.

use serde::{Deserialize, Serialize};

use crate::contracts::identity::data::primitives::{KindId, VersionId};
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};

use super::{
    RelationalAuthorizationObservationEvidence, RelationalAuthorizationTraversalDirection,
};

pub mod capture;
pub mod currentness;
pub mod storage;
pub mod wire;
use std::sync::Arc;
pub use wire::RelationalAuthorizationDependencyWire;

pub const MAXIMUM_AUTHORIZATION_DEPENDENCIES: usize = 4_096;
pub const MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES: usize = 65_536;
const DURABLE_DEPENDENCY_VERSION: u8 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalAuthorizationDependencyDenial {
    Execution(crate::contracts::execution::denial_cause::RelationalExecutionDenialCause),
    Allocation(worth_execution::ExecutionAllocationDenial),
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

#[cfg_attr(not(doctest), doc = include_str!("../../docs/facade_examples/authorization_durable_dependencies.md"))]
#[derive(Clone, Debug)]
pub struct RelationalAuthorizationDurableDependencies {
    owned: Arc<OwnedDependencies>,
}
#[derive(Debug)]
struct OwnedDependencies {
    data: DependencyData,
    _storage: storage::DependencyStorage,
}
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DependencyData {
    version: u8,
    principal: EntityDependency,
    scope: EntityDependency,
    paths: Vec<PathDependencies>,
}
impl PartialEq for RelationalAuthorizationDurableDependencies {
    fn eq(&self, other: &Self) -> bool {
        self.owned.data == other.owned.data
    }
}
impl Eq for RelationalAuthorizationDurableDependencies {}
impl Serialize for RelationalAuthorizationDurableDependencies {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.owned.data.serialize(serializer)
    }
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
    revision: worth_relational_storage::internal_port::storage::data::field_revision::RelationalFieldRevision,
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
        self == fresh && self.owned.data.version == DURABLE_DEPENDENCY_VERSION
    }
}
