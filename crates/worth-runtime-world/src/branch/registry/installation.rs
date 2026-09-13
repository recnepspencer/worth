use std::sync::{Arc, OnceLock};

use crate::branch::ProductBranchReferenceSnapshot;
use crate::identity::{CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation};

/// Exact, monotonic evidence of the registry insertion of one creation.
/// The reservation issues it before forks; only registry insertion fills it.
/// Retirement does not erase the fact, and this witness authorizes no action.
#[derive(Debug)]
pub(crate) struct ProductBranchInstallationWitness {
    branch: ProductBranchIdentity,
    incarnation: ProductBranchIncarnation,
    installed_commit: OnceLock<CompositeCommitIdentity>,
}

impl ProductBranchInstallationWitness {
    pub(super) fn reserve(
        branch: ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
    ) -> Arc<Self> {
        Arc::new(Self {
            branch,
            incarnation,
            installed_commit: OnceLock::new(),
        })
    }

    pub(crate) fn destination(&self) -> (&ProductBranchIdentity, ProductBranchIncarnation) {
        (&self.branch, self.incarnation)
    }

    pub(crate) fn installed_commit(&self) -> Option<&CompositeCommitIdentity> {
        self.installed_commit.get()
    }

    pub(super) fn admits(&self, snapshot: &ProductBranchReferenceSnapshot) -> bool {
        self.installed_commit.get().is_none()
            && snapshot.branch_identity() == &self.branch
            && snapshot.lifecycle_incarnation() == self.incarnation
            && snapshot.reference_generation()
                == crate::identity::ProductBranchReferenceGeneration::initial()
    }

    pub(super) fn record_installation(&self, commit: CompositeCommitIdentity) {
        self.installed_commit
            .set(commit)
            .expect("one admitted creation installs once");
    }

    pub(crate) const fn metadata_charge_hint() -> usize {
        std::mem::size_of::<Self>()
            + std::mem::size_of::<Arc<Self>>()
            + crate::branch::ProductBranchName::MAXIMUM_LENGTH
    }
}
