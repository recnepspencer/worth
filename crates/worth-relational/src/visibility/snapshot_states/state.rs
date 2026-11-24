use std::collections::BTreeMap;

use crate::history::data::BranchId;
use crate::history::retention::SnapshotProjectionRoot;
use crate::identity::data::{PartitionId, VersionId};
use crate::snapshots::data::SnapshotHandle;
use crate::storage::overlay::SnapshotPartitionPins;

use super::{HistoricalVisibilityBasis, VisibilitySnapshotBasis};

/// Exact equivalence identity for one derived visibility state.
///
/// A global commit version is not sufficient because multiple owner branch
/// references may retain that version while selecting independently moving
/// immutable roots. Root identity is issued only by the Relational owner.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
struct ExactVisibilitySnapshotStateKey {
    branch_id: BranchId,
    version_id: VersionId,
    root_id: u64,
    schema_commitment: [u8; 32],
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
struct HistoricalVisibilitySnapshotStateKey {
    branch_id: BranchId,
    version_id: VersionId,
    source_root_id: Option<u64>,
    schema_commitment: Option<[u8; 32]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub(crate) struct VisibilitySnapshotStateKey(VisibilitySnapshotStateKeyKind);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
enum VisibilitySnapshotStateKeyKind {
    Exact(ExactVisibilitySnapshotStateKey),
    Historical(HistoricalVisibilitySnapshotStateKey),
}

impl VisibilitySnapshotStateKey {
    pub(crate) fn for_selection(
        branch_id: BranchId,
        version_id: VersionId,
        root_id: u64,
        schema_commitment: [u8; 32],
    ) -> Self {
        Self(VisibilitySnapshotStateKeyKind::Exact(
            ExactVisibilitySnapshotStateKey {
                branch_id,
                version_id,
                root_id,
                schema_commitment,
            },
        ))
    }

    pub(crate) fn historical(basis: &HistoricalVisibilityBasis) -> Self {
        Self(VisibilitySnapshotStateKeyKind::Historical(
            HistoricalVisibilitySnapshotStateKey {
                branch_id: basis.branch_id().clone(),
                version_id: basis.version_id(),
                source_root_id: basis.source_root_id(),
                schema_commitment: basis.schema_commitment(),
            },
        ))
    }

    pub(crate) fn branch_id(&self) -> &BranchId {
        match &self.0 {
            VisibilitySnapshotStateKeyKind::Exact(key) => &key.branch_id,
            VisibilitySnapshotStateKeyKind::Historical(key) => &key.branch_id,
        }
    }

    pub(crate) const fn version_id(&self) -> VersionId {
        match &self.0 {
            VisibilitySnapshotStateKeyKind::Exact(key) => key.version_id,
            VisibilitySnapshotStateKeyKind::Historical(key) => key.version_id,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum SnapshotStateBasis {
    Exact(VisibilitySnapshotBasis),
    Historical(HistoricalVisibilityBasis),
}

impl SnapshotStateBasis {
    pub(crate) fn key(&self) -> VisibilitySnapshotStateKey {
        match self {
            Self::Exact(basis) => basis.key().clone(),
            Self::Historical(basis) => VisibilitySnapshotStateKey::historical(basis),
        }
    }

    pub(crate) fn branch_id(&self) -> &BranchId {
        match self {
            Self::Exact(basis) => basis.branch_id(),
            Self::Historical(basis) => basis.branch_id(),
        }
    }

    pub(crate) const fn version_id(&self) -> VersionId {
        match self {
            Self::Exact(basis) => basis.version_id(),
            Self::Historical(basis) => basis.version_id(),
        }
    }

    pub(crate) fn root(&self) -> Option<SnapshotProjectionRoot<'_>> {
        match self {
            Self::Exact(basis) => Some(basis.projection_root()),
            Self::Historical(basis) => basis.root(),
        }
    }

    pub(crate) fn root_version(&self) -> Option<VersionId> {
        self.root()
            .and_then(SnapshotProjectionRoot::storage_version)
    }
}

/// Derived pin set plus the exact immutable owner root it was built from.
#[derive(Debug, Clone)]
pub(crate) struct SnapshotState {
    pub(crate) handle: SnapshotHandle,
    pub(crate) basis: SnapshotStateBasis,
    pub(crate) pinned_partitions: BTreeMap<PartitionId, SnapshotPartitionPins>,
    pub(crate) pinned_entity_count: usize,
    pub(crate) pinned_relation_count: usize,
}
