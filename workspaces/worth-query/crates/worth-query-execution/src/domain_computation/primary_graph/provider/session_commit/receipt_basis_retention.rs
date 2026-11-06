//! Bounded owner for basis leases made available to commit receipts.

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use worth_relational::facade::branch::{
    RelationalBranchBasisDescriptor, RelationalBranchRetentionLease,
};
use worth_relational::facade::history::CommitId;

const RETAINED_RECEIPT_BASIS_LIMIT: usize = 64;

#[derive(Clone)]
pub struct WorthQueryRetainedApplicationCommitBasis {
    lease: Arc<RelationalBranchRetentionLease>,
}

impl WorthQueryRetainedApplicationCommitBasis {
    fn new(lease: RelationalBranchRetentionLease) -> Self {
        Self {
            lease: Arc::new(lease),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn descriptor(
        &self,
    ) -> &RelationalBranchBasisDescriptor {
        self.lease.descriptor()
    }

    pub(in crate::domain_computation::primary_graph) fn lease(
        &self,
    ) -> &RelationalBranchRetentionLease {
        &self.lease
    }
}

impl std::fmt::Debug for WorthQueryRetainedApplicationCommitBasis {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryRetainedApplicationCommitBasis")
            .field("descriptor", &self.descriptor())
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryRetainedApplicationCommitBasis {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor() == other.descriptor()
    }
}

impl Eq for WorthQueryRetainedApplicationCommitBasis {}

#[derive(Default)]
pub struct WorthQueryReceiptBasisRetentionStore {
    order: VecDeque<CommitId>,
    by_commit: BTreeMap<CommitId, WorthQueryRetainedApplicationCommitBasis>,
    mandatory: BTreeMap<CommitId, (WorthQueryRetainedApplicationCommitBasis, usize)>,
}

impl WorthQueryReceiptBasisRetentionStore {
    pub(in crate::domain_computation::primary_graph::provider) fn has_mandatory(
        &self,
        commit: CommitId,
    ) -> bool {
        self.mandatory.contains_key(&commit)
    }

    pub(in crate::domain_computation::primary_graph::provider) fn release_mandatory(
        &mut self,
        commit: CommitId,
    ) {
        let (_, claims) = self
            .mandatory
            .get_mut(&commit)
            .expect("terminal release owns its exact dispatch-basis claim");
        *claims = claims.checked_sub(1).expect("a live dispatch claim");
        if *claims == 0 {
            self.mandatory.remove(&commit);
        }
    }
    pub(in crate::domain_computation::primary_graph::provider) fn release(
        &mut self,
        commit: CommitId,
    ) {
        self.by_commit.remove(&commit);
        self.mandatory.remove(&commit);
        self.order.retain(|indexed| *indexed != commit);
    }

    /// A mandatory basis belongs to its dispatch terminal, not to history.
    pub(in crate::domain_computation::primary_graph::provider) fn release_ordinary(
        &mut self,
        commit: CommitId,
    ) {
        if self.by_commit.remove(&commit).is_some() {
            self.order.retain(|indexed| *indexed != commit);
        }
    }

    pub(in crate::domain_computation::primary_graph::provider) fn retain_group(
        &mut self,
        commit: CommitId,
        lease: RelationalBranchRetentionLease,
        dispatch_claims: usize,
    ) {
        let retention = WorthQueryRetainedApplicationCommitBasis::new(lease);
        if dispatch_claims != 0 {
            assert!(
                self.mandatory
                    .insert(commit, (retention, dispatch_claims))
                    .is_none(),
                "one mandatory dispatch basis per commit"
            );
            return;
        }
        assert!(
            self.by_commit.insert(commit, retention).is_none(),
            "one Relational commit may open one receipt-basis lifecycle"
        );
        self.order.push_back(commit);
        while self.order.len() > RETAINED_RECEIPT_BASIS_LIMIT {
            let expired = self
                .order
                .pop_front()
                .expect("an over-capacity receipt-basis lifecycle has an oldest commit");
            self.by_commit.remove(&expired);
        }
    }

    pub(in crate::domain_computation::primary_graph) fn acquire(
        &self,
        commit: CommitId,
    ) -> Option<WorthQueryRetainedApplicationCommitBasis> {
        self.mandatory
            .get(&commit)
            .map(|(basis, _)| basis)
            .or_else(|| self.by_commit.get(&commit))
            .cloned()
    }

    pub(in crate::domain_computation::primary_graph) fn retained_count(&self) -> usize {
        self.by_commit.len() + self.mandatory.len()
    }
}

mod court_doors_for_test;
