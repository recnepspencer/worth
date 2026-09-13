use worth_relational::facade::branch::{AdmittedRelationalBranchBasis, RelationalForkOutcome};
use worth_relational::facade::history::RelationalCommitIdentity;
use worth_relational::facade::publication::DeferredPublicationSettlement;
use worth_relational::facade::transactions::CommitResult;

use super::{CompositeRelationalOwnerResult, CompositeRelationalOwnerResultKind};

impl CompositeRelationalOwnerResult {
    pub(crate) fn retained() -> Self {
        Self {
            result: CompositeRelationalOwnerResultKind::RetainedExact,
        }
    }

    pub(crate) fn settled(
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        result: std::sync::Arc<CommitResult>,
    ) -> Self {
        Self {
            result: CompositeRelationalOwnerResultKind::Published {
                commit_identity,
                successor_basis,
                settlement: Some(result.outcome().commit.clone()),
                result: Some(result),
            },
        }
    }

    pub(crate) fn settlement_pending(
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        settlement: DeferredPublicationSettlement,
    ) -> Self {
        Self {
            result: CompositeRelationalOwnerResultKind::SettlementPending {
                commit_identity,
                successor_basis,
                _settlement: settlement,
            },
        }
    }

    pub(crate) fn settlement_required(
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            result: CompositeRelationalOwnerResultKind::SettlementRequired {
                commit_identity,
                successor_basis,
            },
        }
    }

    pub(crate) fn settled_receipt(
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        receipt: worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Self {
        Self {
            result: CompositeRelationalOwnerResultKind::Published {
                commit_identity,
                successor_basis,
                settlement: Some(receipt),
                result: None,
            },
        }
    }

    /// The only Relational fork result. Creation forks; publication does not,
    /// so a fork never shares a result with a commit.
    pub(crate) fn forked(
        fork: RelationalForkOutcome,
        successor_basis: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            result: CompositeRelationalOwnerResultKind::Forked {
                fork,
                successor_basis,
            },
        }
    }
}
