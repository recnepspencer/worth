use crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity;
use std::sync::Arc;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_runtime_bridge::facade::TruthBranchIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphWorkBranchAffinity {
    relational: BranchId,
    truth: TruthBranchIdentity,
}

impl WorthQueryGraphWorkBranchAffinity {
    pub(super) fn from_query_basis(basis: &WorthQueryApplicationBasisIdentity) -> Self {
        Self::from_relational_branch(basis.branch_id().clone())
    }

    pub(super) fn from_snapshot(snapshot: &SnapshotHandle) -> Self {
        Self::from_relational_branch(snapshot.branch_id().clone())
    }

    pub(super) fn from_product(product: &crate::basis::WorthQueryProductObservationLease) -> Self {
        Self::from_relational_branch(product.relational_basis_descriptor().branch_id().clone())
    }

    fn from_relational_branch(relational: BranchId) -> Self {
        let truth =
            TruthBranchIdentity::from_relational_branch_id(Arc::<str>::from(relational.0.as_str()));
        Self { relational, truth }
    }

    pub(in crate::domain_computation) const fn relational(&self) -> &BranchId {
        &self.relational
    }

    pub(in crate::domain_computation) const fn truth(&self) -> &TruthBranchIdentity {
        &self.truth
    }

    pub(super) fn admits_query_basis(&self, basis: &WorthQueryApplicationBasisIdentity) -> bool {
        basis.branch_id() == &self.relational
    }

    pub(super) fn admits_snapshot(&self, snapshot: &SnapshotHandle) -> bool {
        snapshot.branch_id() == &self.relational
    }
}

mod court_doors_for_test;
