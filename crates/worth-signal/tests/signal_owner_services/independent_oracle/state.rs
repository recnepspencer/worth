use std::collections::BTreeMap;

/// Numeric identity is intentionally the only branch identity the model
/// needs. The production branch-name encoding is not copied into the oracle.
pub(crate) type BranchKey = u64;
pub(crate) type LeaseKey = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelOwnerLifecycle {
    Open,
    Closing,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelBranchLifecycle {
    Live,
    Retired,
}

/// Neutral state extracted from a public admitted basis.
///
/// This is not a copy of Signal's basis or target type. It contains only the
/// values needed to compare the observable branch contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelObservation {
    pub(crate) branch: BranchKey,
    pub(crate) graph_instance: String,
    pub(crate) definition_basis: u64,
    pub(crate) snapshot: Option<u64>,
    pub(crate) restore_snapshot: Option<u64>,
    pub(crate) generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelSnapshot {
    pub(crate) branch: BranchKey,
    pub(crate) snapshot: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelBranch {
    pub(crate) key: BranchKey,
    pub(crate) parent: Option<BranchKey>,
    pub(crate) name: String,
    pub(crate) observation: ModelObservation,
    pub(crate) lifecycle: ModelBranchLifecycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelLease {
    pub(crate) branch: BranchKey,
    pub(crate) observation: ModelObservation,
}

#[derive(Debug, Clone)]
pub(crate) struct ModelWorld {
    pub(crate) lifecycle: ModelOwnerLifecycle,
    pub(crate) lifecycle_history: Vec<ModelOwnerLifecycle>,
    pub(crate) root: BranchKey,
    pub(crate) next_lease: LeaseKey,
    pub(crate) branches: BTreeMap<BranchKey, ModelBranch>,
    pub(crate) leases: BTreeMap<LeaseKey, ModelLease>,
}

impl ModelWorld {
    pub(crate) fn bootstrap(
        root: BranchKey,
        name: impl Into<String>,
        observation: ModelObservation,
    ) -> Self {
        let branch = ModelBranch {
            key: root,
            parent: None,
            name: name.into(),
            observation,
            lifecycle: ModelBranchLifecycle::Live,
        };
        Self {
            lifecycle: ModelOwnerLifecycle::Open,
            lifecycle_history: vec![ModelOwnerLifecycle::Open],
            root,
            next_lease: 1,
            branches: BTreeMap::from([(root, branch)]),
            leases: BTreeMap::new(),
        }
    }

    pub(crate) fn branch(&self, key: BranchKey) -> Option<&ModelBranch> {
        self.branches.get(&key)
    }

    pub(crate) fn live_branch(&self, key: BranchKey) -> Option<&ModelBranch> {
        self.branch(key)
            .filter(|branch| branch.lifecycle == ModelBranchLifecycle::Live)
    }

    pub(crate) fn branch_mut(&mut self, key: BranchKey) -> Option<&mut ModelBranch> {
        self.branches.get_mut(&key)
    }

    pub(crate) fn has_retention_for(&self, key: BranchKey) -> bool {
        self.leases.values().any(|lease| lease.branch == key)
    }

    pub(crate) fn add_lease(
        &mut self,
        branch: BranchKey,
        observation: ModelObservation,
    ) -> LeaseKey {
        let key = self.next_lease;
        self.next_lease += 1;
        self.leases.insert(
            key,
            ModelLease {
                branch,
                observation,
            },
        );
        key
    }

    pub(crate) fn close(&mut self) {
        if self.lifecycle == ModelOwnerLifecycle::Open {
            self.lifecycle = ModelOwnerLifecycle::Closing;
            self.lifecycle_history.push(ModelOwnerLifecycle::Closing);
            self.lifecycle = ModelOwnerLifecycle::Closed;
            self.lifecycle_history.push(ModelOwnerLifecycle::Closed);
        }
    }
}
