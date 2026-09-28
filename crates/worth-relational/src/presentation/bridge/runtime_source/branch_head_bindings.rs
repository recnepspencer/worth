use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use worth_runtime_bridge::facade::{
    RelationalBridgeSourceError, TruthBranchIdentity, TruthSnapshotIdentity,
};

use crate::facade::history::CommitId;

use super::{RelationalBridgeObservationLease, RelationalBridgeObservationReleaseReceipt};

#[derive(Debug, Default)]
pub(super) struct RelationalBridgeBranchHeadBindings {
    next_binding_id: AtomicU64,
    entries: Mutex<BTreeMap<TruthBranchIdentity, RelationalBridgeBranchHeadBinding>>,
}

#[derive(Debug)]
struct RelationalBridgeBranchHeadBinding {
    binding_id: u64,
    commit_id: CommitId,
    snapshot_identity: TruthSnapshotIdentity,
}

impl RelationalBridgeBranchHeadBindings {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub(super) fn insert(
        self: &Arc<Self>,
        branch_identity: TruthBranchIdentity,
        commit_id: CommitId,
        observation: RelationalBridgeObservationLease,
    ) -> RelationalBridgeBranchHeadLease {
        let binding_id = self.next_binding_id.fetch_add(1, Ordering::Relaxed);
        let snapshot_identity = observation.snapshot_identity().clone();
        self.lock_entries().insert(
            branch_identity.clone(),
            RelationalBridgeBranchHeadBinding {
                binding_id,
                commit_id,
                snapshot_identity,
            },
        );
        RelationalBridgeBranchHeadLease {
            branch_identity,
            binding_id,
            bindings: Some(Arc::clone(self)),
            observation: Some(observation),
        }
    }

    pub(super) fn resolve(
        &self,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<(CommitId, TruthSnapshotIdentity), RelationalBridgeSourceError> {
        let entries = self.lock_entries();
        let binding = entries.get(branch_identity).ok_or_else(|| {
            RelationalBridgeSourceError::new(format!(
                "relational Bridge branch `{branch_identity:?}` has no explicitly admitted head basis"
            ))
        })?;
        Ok((binding.commit_id, binding.snapshot_identity.clone()))
    }

    pub(super) fn unique_snapshot_for_commit(
        &self,
        commit_id: CommitId,
    ) -> Result<Option<TruthSnapshotIdentity>, RelationalBridgeSourceError> {
        let entries = self.lock_entries();
        let mut matching = entries
            .values()
            .filter(|binding| binding.commit_id == commit_id)
            .map(|binding| binding.snapshot_identity.clone());
        let first = matching.next();
        if matching.next().is_some() {
            return Err(RelationalBridgeSourceError::new(format!(
                "relational commit `{}` is the admitted head of multiple branches; an exact branch-head request is required",
                commit_id.0
            )));
        }
        Ok(first)
    }

    fn remove(&self, branch_identity: &TruthBranchIdentity, binding_id: u64) -> bool {
        let mut entries = self.lock_entries();
        if entries
            .get(branch_identity)
            .is_some_and(|binding| binding.binding_id == binding_id)
        {
            entries.remove(branch_identity);
            true
        } else {
            false
        }
    }

    fn lock_entries(
        &self,
    ) -> MutexGuard<'_, BTreeMap<TruthBranchIdentity, RelationalBridgeBranchHeadBinding>> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Move-only registration of one owner-admitted basis as a Bridge branch head.
#[derive(Debug)]
pub struct RelationalBridgeBranchHeadLease {
    branch_identity: TruthBranchIdentity,
    binding_id: u64,
    bindings: Option<Arc<RelationalBridgeBranchHeadBindings>>,
    observation: Option<RelationalBridgeObservationLease>,
}

impl RelationalBridgeBranchHeadLease {
    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        self.observation
            .as_ref()
            .expect("live branch-head lease carries its observation")
            .snapshot_identity()
    }

    pub fn release(mut self) -> RelationalBridgeBranchHeadReleaseReceipt {
        let unbound = self
            .bindings
            .take()
            .is_some_and(|bindings| bindings.remove(&self.branch_identity, self.binding_id));
        let observation = self
            .observation
            .take()
            .expect("branch-head lease can be released only once")
            .release();
        RelationalBridgeBranchHeadReleaseReceipt {
            branch_identity: self.branch_identity.clone(),
            observation,
            unbound,
        }
    }
}

impl Drop for RelationalBridgeBranchHeadLease {
    fn drop(&mut self) {
        if let Some(bindings) = self.bindings.take() {
            let _ = bindings.remove(&self.branch_identity, self.binding_id);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalBridgeBranchHeadReleaseReceipt {
    branch_identity: TruthBranchIdentity,
    observation: RelationalBridgeObservationReleaseReceipt,
    unbound: bool,
}

impl RelationalBridgeBranchHeadReleaseReceipt {
    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }

    pub fn observation(&self) -> &RelationalBridgeObservationReleaseReceipt {
        &self.observation
    }

    pub const fn unbound(&self) -> bool {
        self.unbound
    }
}
