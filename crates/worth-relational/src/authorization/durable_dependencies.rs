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
mod currentness;
mod storage;
mod wire;
use std::sync::Arc;
pub use wire::RelationalAuthorizationDependencyWire;

pub const MAXIMUM_AUTHORIZATION_DEPENDENCIES: usize = 4_096;
pub const MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES: usize = 65_536;
const DURABLE_DEPENDENCY_VERSION: u8 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalAuthorizationDependencyDenial {
    Execution(crate::execution::RelationalExecutionDenialCause),
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

/// Issued/decoded native description with shared deep-storage custody.
/// Context-free deserialization cannot construct this owning value.
///
/// ```compile_fail
/// use worth_relational::facade::authorization::RelationalAuthorizationDurableDependencies;
/// fn unpaid(bytes: &[u8]) -> RelationalAuthorizationDurableDependencies {
///     rmp_serde::from_slice(bytes).unwrap()
/// }
/// ```
///
/// ```
/// use worth_relational::facade::authorization::{RelationalAuthorizationDurableDependencies,
///     RelationalAuthorizationDependencyDenial};
/// fn admitted(execution: &worth_execution::ExecutionRequest<'_, '_>, wire: &str)
///     -> Result<RelationalAuthorizationDurableDependencies, RelationalAuthorizationDependencyDenial> {
///     RelationalAuthorizationDurableDependencies::from_wire(execution, wire)
/// }
/// ```
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
        self == fresh && self.owned.data.version == DURABLE_DEPENDENCY_VERSION
    }
}

impl RelationalRuntime {
    /// Capture exact source revisions from the evidence's own immutable root.
    /// Cost is O(tracked reads); no graph reconstruction is permitted here.
    pub fn capture_authorization_durable_dependencies(
        &self,
        evidence: &RelationalAuthorizationObservationEvidence,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthorizationDurableDependencies, RelationalAuthorizationDependencyDenial>
    {
        capture::capture(self, execution, evidence)
    }
}
