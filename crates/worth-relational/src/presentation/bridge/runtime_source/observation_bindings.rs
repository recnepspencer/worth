use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

use worth_runtime_bridge::facade::{RelationalBridgeSourceError, TruthSnapshotIdentity};

use crate::history::data::CommitId;
use crate::history::retention::RelationalBranchRetentionLease;
use crate::identity::data::VersionId;
use crate::mvcc::RelationalBranchObservation;
use crate::snapshots::data::SnapshotId;

#[derive(Clone, Debug)]
pub(in crate::presentation::bridge) struct RelationalBridgeSelectedObservation {
    pub(super) snapshot_identity: TruthSnapshotIdentity,
    pub(super) observation: RelationalBranchObservation,
}

#[derive(Debug)]
pub(in crate::presentation::bridge) struct RelationalBridgeSelectedCommitObservation {
    pub(super) commit_id: CommitId,
    pub(super) observation: RelationalBridgeSelectedObservation,
}

impl RelationalBridgeSelectedObservation {
    pub(in crate::presentation::bridge) fn branch_id(&self) -> &crate::history::data::BranchId {
        self.observation.identity().branch_id()
    }

    pub(super) fn observation(&self) -> &RelationalBranchObservation {
        &self.observation
    }
}

impl RelationalBridgeSelectedCommitObservation {
    pub(in crate::presentation::bridge) fn into_parts(
        self,
    ) -> (
        CommitId,
        TruthSnapshotIdentity,
        crate::history::data::BranchId,
    ) {
        (
            self.commit_id,
            self.observation.snapshot_identity,
            self.observation.observation.identity().branch_id().clone(),
        )
    }
}

#[derive(Debug, Default)]
pub(super) struct RelationalBridgeObservationBindings {
    entries: Mutex<RelationalBridgeObservationBindingIndex>,
}

#[derive(Debug, Default)]
struct RelationalBridgeObservationBindingIndex {
    by_snapshot: HashMap<SnapshotId, RelationalBridgeObservationBinding>,
    by_commit: HashMap<CommitId, HashSet<SnapshotId>>,
}

#[derive(Debug)]
struct RelationalBridgeObservationBinding {
    version_id: VersionId,
    commit_id: Option<CommitId>,
    observation: RelationalBranchObservation,
    retention: RelationalBranchRetentionLease,
}

impl RelationalBridgeObservationBindings {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub(super) fn insert(
        self: &Arc<Self>,
        snapshot_id: SnapshotId,
        observation: RelationalBranchObservation,
        retention: RelationalBranchRetentionLease,
    ) -> RelationalBridgeObservationLease {
        let version_id = observation.version_id();
        let commit_id = observation.commit_id();
        let mut entries = self.lock_entries();
        entries.by_snapshot.insert(
            snapshot_id,
            RelationalBridgeObservationBinding {
                version_id,
                commit_id,
                observation,
                retention,
            },
        );
        if let Some(commit_id) = commit_id {
            entries
                .by_commit
                .entry(commit_id)
                .or_default()
                .insert(snapshot_id);
        }
        drop(entries);
        let snapshot_identity =
            crate::presentation::bridge::identities::bridge_snapshot_identity_for_binding(
                snapshot_id,
                version_id,
            );
        RelationalBridgeObservationLease {
            snapshot_identity,
            snapshot_id,
            bindings: Some(Arc::clone(self)),
        }
    }

    pub(super) fn resolve(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<RelationalBridgeSelectedObservation, RelationalBridgeSourceError> {
        let (snapshot_id, expected_version_id) =
            crate::presentation::bridge::identities::parse_bridge_snapshot_identity(identity)?;
        let entries = self.lock_entries();
        let binding = entries.by_snapshot.get(&snapshot_id).ok_or_else(|| {
            RelationalBridgeSourceError::new(format!(
                "relational bridge snapshot `{}` has no retained owner-admitted observation",
                snapshot_id.0
            ))
        })?;
        if binding.version_id != expected_version_id {
            return Err(RelationalBridgeSourceError::new(format!(
                "relational bridge observation `{}` expected version `{}` but retained basis selects version `{}`",
                snapshot_id.0, expected_version_id.0, binding.version_id.0
            )));
        }
        Ok(RelationalBridgeSelectedObservation {
            snapshot_identity: identity.clone(),
            observation: binding.observation.clone(),
        })
    }

    pub(super) fn snapshot_identity_for_commit(
        &self,
        commit_id: CommitId,
    ) -> Result<TruthSnapshotIdentity, RelationalBridgeSourceError> {
        let entries = self.lock_entries();
        let snapshot_ids = entries.by_commit.get(&commit_id).ok_or_else(|| {
            RelationalBridgeSourceError::new(format!(
                "relational commit `{}` has no retained owner-admitted Bridge observation",
                commit_id.0
            ))
        })?;
        if snapshot_ids.len() != 1 {
            return Err(RelationalBridgeSourceError::new(format!(
                "relational commit `{}` has multiple admitted Bridge observations; an exact branch-head binding is required",
                commit_id.0
            )));
        }
        let snapshot_id = *snapshot_ids
            .iter()
            .next()
            .expect("non-empty commit binding set has one snapshot");
        let binding = entries
            .by_snapshot
            .get(&snapshot_id)
            .expect("commit index references a live observation binding");
        Ok(
            crate::presentation::bridge::identities::bridge_snapshot_identity_for_binding(
                snapshot_id,
                binding.version_id,
            ),
        )
    }

    fn remove(&self, snapshot_id: SnapshotId) -> Option<RelationalBridgeObservationBinding> {
        let mut entries = self.lock_entries();
        let binding = entries.by_snapshot.remove(&snapshot_id)?;
        if let Some(commit_id) = binding.commit_id {
            let remove_commit = entries
                .by_commit
                .get_mut(&commit_id)
                .is_some_and(|snapshots| {
                    snapshots.remove(&snapshot_id);
                    snapshots.is_empty()
                });
            if remove_commit {
                entries.by_commit.remove(&commit_id);
            }
        }
        Some(binding)
    }

    fn lock_entries(&self) -> MutexGuard<'_, RelationalBridgeObservationBindingIndex> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Move-only Bridge registration for one concrete Relational observation.
#[derive(Debug)]
pub struct RelationalBridgeObservationLease {
    snapshot_identity: TruthSnapshotIdentity,
    snapshot_id: SnapshotId,
    bindings: Option<Arc<RelationalBridgeObservationBindings>>,
}

impl RelationalBridgeObservationLease {
    /// Confirm that this retained Bridge observation was issued from the exact
    /// Relational basis carried by a composite product observation.
    pub fn admits_basis(&self, basis: &crate::branch::AdmittedRelationalBranchBasis) -> bool {
        self.bindings
            .as_ref()
            .and_then(|bindings| self.resolve_for(bindings).ok())
            .is_some_and(|selected| selected.observation().admitted_basis() == *basis)
    }

    pub(super) fn resolve_for(
        &self,
        bindings: &Arc<RelationalBridgeObservationBindings>,
    ) -> Result<RelationalBridgeSelectedObservation, RelationalBridgeSourceError> {
        if !self
            .bindings
            .as_ref()
            .is_some_and(|issuer| Arc::ptr_eq(issuer, bindings))
        {
            return Err(RelationalBridgeSourceError::new(
                "retained Bridge observation belongs to another source registration owner",
            ));
        }
        bindings.resolve(&self.snapshot_identity)
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn release(mut self) -> RelationalBridgeObservationReleaseReceipt {
        let component_release = self
            .bindings
            .take()
            .and_then(|bindings| bindings.remove(self.snapshot_id))
            .map(|binding| binding.retention.release());
        RelationalBridgeObservationReleaseReceipt {
            snapshot_identity: self.snapshot_identity.clone(),
            component_release,
        }
    }
}

impl Drop for RelationalBridgeObservationLease {
    fn drop(&mut self) {
        if let Some(bindings) = self.bindings.take() {
            let _ = bindings.remove(self.snapshot_id);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalBridgeObservationReleaseReceipt {
    snapshot_identity: TruthSnapshotIdentity,
    component_release: Option<crate::history::retention::RelationalBranchRetentionReleaseReceipt>,
}

impl RelationalBridgeObservationReleaseReceipt {
    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub const fn released(&self) -> bool {
        self.component_release.is_some()
    }

    pub fn component_release(
        &self,
    ) -> Option<&crate::history::retention::RelationalBranchRetentionReleaseReceipt> {
        self.component_release.as_ref()
    }
}
