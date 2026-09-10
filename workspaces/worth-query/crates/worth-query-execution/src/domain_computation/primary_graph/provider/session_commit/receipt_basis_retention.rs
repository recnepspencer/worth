//! Bounded owner for basis leases made available to commit receipts.

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use worth_relational::facade::branch::{
    RelationalBranchBasisDescriptor, RelationalBranchRetentionLease,
};
use worth_relational::facade::history::CommitId;

const RETAINED_RECEIPT_BASIS_LIMIT: usize = 64;

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryRetainedApplicationCommitBasis {
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
pub(in crate::domain_computation::primary_graph) struct WorthQueryReceiptBasisRetentionStore {
    order: VecDeque<CommitId>,
    by_commit: BTreeMap<CommitId, WorthQueryRetainedApplicationCommitBasis>,
}

impl WorthQueryReceiptBasisRetentionStore {
    pub(in crate::domain_computation::primary_graph::provider) fn release(
        &mut self,
        commit: CommitId,
    ) {
        self.by_commit.remove(&commit);
        self.order.retain(|indexed| *indexed != commit);
    }

    pub(in crate::domain_computation::primary_graph::provider) fn retain(
        &mut self,
        commit: CommitId,
        lease: RelationalBranchRetentionLease,
    ) {
        let retention = WorthQueryRetainedApplicationCommitBasis::new(lease);
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
        self.by_commit.get(&commit).cloned()
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn retained_count(&self) -> usize {
        self.by_commit.len()
    }
}
