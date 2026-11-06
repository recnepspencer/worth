use worth_relational::facade::branch::{AdmittedRelationalBranchBasis, RelationalForkOutcome};
use worth_relational::facade::history::RelationalCommitIdentity;
use worth_relational::facade::transactions::CommitResult;

use super::{
    CompositeOwnerExecutionResults, CompositeRelationalOwnerResult,
    CompositeRelationalOwnerResultKind,
};

impl CompositeOwnerExecutionResults {
    /// A terminal no-effect release requires the actual closed owner result.
    pub(crate) fn release_original_component(
        &self,
    ) -> Result<
        Option<(
            &worth_relational::facade::history::RelationalCommitReceipt,
            &AdmittedRelationalBranchBasis,
        )>,
        (),
    > {
        match &self.relational.result {
            CompositeRelationalOwnerResultKind::RetainedExact
            | CompositeRelationalOwnerResultKind::Forked { .. } => Ok(None),
            CompositeRelationalOwnerResultKind::Published {
                settlement: Some(receipt),
                successor_basis: Some(basis),
                ..
            } => Ok(Some((receipt, basis))),
            CompositeRelationalOwnerResultKind::Published {
                settlement: None, ..
            }
            | CompositeRelationalOwnerResultKind::Published {
                successor_basis: None,
                ..
            }
            | CompositeRelationalOwnerResultKind::SettlementRequired { .. }
            | CompositeRelationalOwnerResultKind::DurabilityUnresolved { .. } => Err(()),
        }
    }
    /// Absence is coherent only for a leg that creates no commit snapshot.
    pub(crate) fn result_for_snapshot_cleanup(
        &self,
    ) -> Result<Option<std::sync::Arc<CommitResult>>, ()> {
        match &self.relational.result {
            CompositeRelationalOwnerResultKind::RetainedExact
            | CompositeRelationalOwnerResultKind::Forked { .. } => Ok(None),
            CompositeRelationalOwnerResultKind::Published {
                settlement: Some(_),
                result: Some(result),
                ..
            } => Ok(Some(result.clone())),
            CompositeRelationalOwnerResultKind::Published { .. }
            | CompositeRelationalOwnerResultKind::SettlementRequired { .. }
            | CompositeRelationalOwnerResultKind::DurabilityUnresolved { .. } => Err(()),
        }
    }
}

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
                successor_basis: Some(successor_basis),
                settlement: Some(result.outcome().commit.clone()),
                result: Some(result),
            },
        }
    }

    /// The owner could not establish whether its commit became durable.
    pub(crate) fn durability_unresolved(
        unresolved: worth_relational::facade::durability::RelationalDurabilityUnresolved,
    ) -> Self {
        Self {
            result: CompositeRelationalOwnerResultKind::DurabilityUnresolved {
                _unresolved: unresolved,
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
                successor_basis: Some(successor_basis),
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
                successor_basis: Some(successor_basis),
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
                successor_basis: Some(successor_basis),
            },
        }
    }
}
