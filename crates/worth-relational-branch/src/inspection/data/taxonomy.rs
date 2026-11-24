use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InspectionOrigin {
    CurrentTruth,
    VisibilitySnapshot,
    CanonicalCommitStorage,
    LineageGraph,
    RetentionState,
    TransactionStaging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InspectionAccessPath {
    DirectLookup,
    SnapshotRead,
    VersionRead,
    HistoricalRetainedRead,
    HistoricalReconstructedRead,
    CommitIndexRead,
    GraphTraversal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InspectionResolutionContext {
    NoContext,
    BranchAncestry,
    LineageTraversal,
    RelationNeighborhood,
    ConnectivityTraversal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InspectionAvailability {
    Direct,
    Reconstructed,
    UnavailableByBudget,
    UnavailableByRetention,
    UnavailableByPolicy,
    UnavailableByMissingCanonicalArtifacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InspectionDegradation {
    MissingStructuralFingerprint,
    MissingLineageIdentity,
    SummaryOnly,
    WorkBudgetExceeded,
    EntityBudgetExceeded,
    RelationBudgetExceeded,
    FrontierBudgetExceeded,
    ComponentBudgetExceeded,
    EntitySlotBudgetExceeded,
    RelationSlotBudgetExceeded,
    ReconstructionOmittedByMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InspectionRecordClass {
    Entity,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectionScope {
    Current,
    Version(VersionId),
    Snapshot(SnapshotHandle),
}

impl Serialize for InspectionScope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Current => InspectionScopeWireRef::Current.serialize(serializer),
            Self::Version(version) => {
                InspectionScopeWireRef::Version(*version).serialize(serializer)
            }
            Self::Snapshot(handle) => {
                InspectionScopeWireRef::Snapshot(SnapshotScopeDescriptorRef {
                    runtime_instance_id: handle.runtime_instance_id(),
                    branch_id: handle.branch_id(),
                    snapshot_id: handle.snapshot_id(),
                    version_id: handle.version_id(),
                    read_policy: handle.read_policy(),
                })
                .serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for InspectionScope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match InspectionScopeWire::deserialize(deserializer)? {
            InspectionScopeWire::Current => Ok(Self::Current),
            InspectionScopeWire::Version(version) => Ok(Self::Version(version)),
            InspectionScopeWire::Snapshot(descriptor) => {
                let SnapshotScopeDescriptor {
                    runtime_instance_id,
                    branch_id,
                    snapshot_id,
                    version_id,
                    read_policy,
                } = descriptor;
                let _ = (
                    runtime_instance_id,
                    branch_id,
                    snapshot_id,
                    version_id,
                    read_policy,
                );
                Err(serde::de::Error::custom(
                    "transported snapshot descriptors require owner readmission",
                ))
            }
        }
    }
}

#[derive(Serialize)]
enum InspectionScopeWireRef<'a> {
    Current,
    Version(VersionId),
    Snapshot(SnapshotScopeDescriptorRef<'a>),
}

#[derive(Serialize)]
struct SnapshotScopeDescriptorRef<'a> {
    runtime_instance_id: u64,
    branch_id: &'a BranchId,
    snapshot_id: SnapshotId,
    version_id: VersionId,
    read_policy: SnapshotReadPolicy,
}

#[derive(Deserialize)]
enum InspectionScopeWire {
    Current,
    Version(VersionId),
    Snapshot(SnapshotScopeDescriptor),
}

#[derive(Deserialize)]
struct SnapshotScopeDescriptor {
    runtime_instance_id: u64,
    branch_id: BranchId,
    snapshot_id: SnapshotId,
    version_id: VersionId,
    read_policy: SnapshotReadPolicy,
}
