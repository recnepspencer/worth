use std::collections::BTreeMap;
use std::sync::Arc;

use crate::branch::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::identity::data::{PartitionId, VersionId};
use crate::snapshots::data::SnapshotHandle;
use crate::storage::overlay::SnapshotPartitionPins;

use super::HistoricalVisibilityBasis;

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
    fn exact(branch_id: BranchId, version_id: VersionId, root: &RelationalBranchRoot) -> Self {
        Self(VisibilitySnapshotStateKeyKind::Exact(
            ExactVisibilitySnapshotStateKey {
                branch_id,
                version_id,
                root_id: root.id(),
                schema_commitment: root.schema_authority().registry().authority_digest_bytes(),
            },
        ))
    }

    pub(crate) fn for_root(
        branch_id: BranchId,
        version_id: VersionId,
        root: &RelationalBranchRoot,
    ) -> Self {
        Self::exact(branch_id, version_id, root)
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

/// Owner-selected immutable root retained by a snapshot handle binding.
#[derive(Clone, Debug)]
pub(crate) struct VisibilitySnapshotBasis {
    key: VisibilitySnapshotStateKey,
    root: Arc<RelationalBranchRoot>,
    _retained_observation: crate::mvcc::RelationalBranchObservation,
}

impl VisibilitySnapshotBasis {
    pub(crate) fn from_observation(observation: &crate::mvcc::RelationalBranchObservation) -> Self {
        let version_id = observation.version_id();
        let root = Arc::clone(observation.selected_root());
        Self {
            key: VisibilitySnapshotStateKey::exact(
                observation.identity().branch_id().clone(),
                version_id,
                root.as_ref(),
            ),
            root,
            _retained_observation: observation.clone(),
        }
    }

    pub(crate) fn capture_current(
        runtime: &crate::runtime::RelationalRuntime,
        branch_id: &BranchId,
        version_id: VersionId,
    ) -> Result<Option<Self>, crate::branch::RelationalBranchBasisDenial> {
        let identity = runtime
            .branch_identity(branch_id)
            .map_err(branch_identity_denial)?;
        let (_, basis) = runtime.observe_branch(&identity)?;
        let observation = basis.observation();
        if observation.version_id() != version_id {
            return Ok(None);
        }
        Ok(Some(Self::from_observation(&observation)))
    }

    /// Best-effort selection for optional derived-state maintenance. The
    /// retention owner records any admission denial; authoritative truth and
    /// branch-head movement never depend on this cache-only promotion.
    pub(crate) fn capture_current_for_optional_maintenance(
        runtime: &crate::runtime::RelationalRuntime,
        branch_id: &BranchId,
        version_id: VersionId,
    ) -> Option<Self> {
        Self::capture_current(runtime, branch_id, version_id)
            .ok()
            .flatten()
    }

    pub(crate) fn key(&self) -> &VisibilitySnapshotStateKey {
        &self.key
    }

    pub(crate) fn branch_id(&self) -> &BranchId {
        self.key.branch_id()
    }

    pub(crate) const fn version_id(&self) -> VersionId {
        self.key.version_id()
    }

    pub(crate) fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }

    pub(crate) fn root_id(&self) -> u64 {
        self.root.id()
    }
}

fn branch_identity_denial(
    denial: crate::branch::RelationalBranchIdentityDenial,
) -> crate::branch::RelationalBranchBasisDenial {
    match denial {
        crate::branch::RelationalBranchIdentityDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        } => crate::branch::RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        },
        crate::branch::RelationalBranchIdentityDenial::UnknownBranch(branch_id) => {
            crate::branch::RelationalBranchBasisDenial::UnknownBranch(branch_id)
        }
        crate::branch::RelationalBranchIdentityDenial::IdentityMismatch => {
            crate::branch::RelationalBranchBasisDenial::MixedAxis(
                crate::branch::RelationalBranchBasisMismatchAxis::Branch,
            )
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

    pub(crate) fn root(&self) -> Option<&Arc<RelationalBranchRoot>> {
        match self {
            Self::Exact(basis) => Some(basis.root()),
            Self::Historical(basis) => basis.root(),
        }
    }

    pub(crate) fn root_id(&self) -> Option<u64> {
        match self {
            Self::Exact(basis) => Some(basis.root_id()),
            Self::Historical(basis) => basis.source_root_id(),
        }
    }

    pub(crate) fn root_version(&self) -> Option<VersionId> {
        self.root()
            .and_then(|root| root.axes())
            .map(|axes| VersionId(axes.storage_version))
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
