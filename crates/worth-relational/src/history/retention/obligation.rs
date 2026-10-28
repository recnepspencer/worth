use std::sync::Arc;

use crate::branch::RelationalBranchRoot;

/// Why an immutable branch root remains retained by the Relational owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalBasisRetentionReason {
    Observation,
    ExternalComponentBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalRetentionObligationKind {
    Observation,
    Transaction,
    Candidate,
    PerformedSettlement,
    ExternalComponentBasis,
}

/// One owner-issued observation obligation shared by all clones of an
/// admitted basis and its repeatable observations.
#[derive(Debug)]
pub(crate) struct RelationalObservationRetentionObligation {
    _guard: super::RelationalRetentionGuard,
}

impl RelationalObservationRetentionObligation {
    pub(crate) fn acquire(
        binding: &super::RelationalBranchRetentionBinding,
        root: Arc<RelationalBranchRoot>,
    ) -> Result<Self, super::RelationalRetentionAcquisitionDenial> {
        binding
            .acquire(
                RelationalRetentionObligationKind::Observation,
                vec![root],
                None,
            )
            .map(|guard| Self { _guard: guard })
    }

    pub(crate) const fn reason(&self) -> RelationalBasisRetentionReason {
        RelationalBasisRetentionReason::Observation
    }
}

#[derive(Debug)]
pub(crate) struct RelationalRetainedHistoricalRoot {
    root: Arc<RelationalBranchRoot>,
    _obligation: RelationalObservationRetentionObligation,
}

impl RelationalRetainedHistoricalRoot {
    pub(crate) fn acquire(
        binding: &super::RelationalBranchRetentionBinding,
        root: Arc<RelationalBranchRoot>,
    ) -> Result<Self, super::RelationalRetentionAcquisitionDenial> {
        let obligation =
            RelationalObservationRetentionObligation::acquire(binding, Arc::clone(&root))?;
        Ok(Self {
            root,
            _obligation: obligation,
        })
    }

    pub(crate) fn from_owner(
        owner: &super::RelationalBranchRetentionOwner,
        commit_id: crate::history::data::CommitId,
    ) -> Result<Option<Self>, super::RelationalRetentionAcquisitionDenial> {
        owner
            .acquire_retired_observation(commit_id)
            .map(|retained| {
                retained.map(|(root, guard)| Self {
                    root,
                    _obligation: RelationalObservationRetentionObligation { _guard: guard },
                })
            })
    }

    pub(crate) fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }
}

/// One transaction-lifetime hold over the exact immutable basis from which
/// detached work was admitted.
#[derive(Debug)]
pub(crate) struct RelationalTransactionRetentionObligation {
    _guard: super::RelationalRetentionGuard,
}

impl RelationalTransactionRetentionObligation {
    pub(crate) fn acquire(
        binding: &super::RelationalBranchRetentionBinding,
        identity: crate::branch::RelationalBranchIdentity,
        root: Arc<RelationalBranchRoot>,
    ) -> Result<Self, super::RelationalRetentionAcquisitionDenial> {
        binding
            .acquire(
                RelationalRetentionObligationKind::Transaction,
                vec![root],
                Some(identity),
            )
            .map(|guard| Self { _guard: guard })
    }
}

/// One prepared-candidate hold over both the comparison root and the complete
/// root that may replace it.
#[derive(Debug)]
pub(crate) struct RelationalCandidateRetentionObligation {
    guard: super::RelationalRetentionGuard,
}

impl RelationalCandidateRetentionObligation {
    pub(crate) fn acquire(
        binding: &super::RelationalBranchRetentionBinding,
        identity: crate::branch::RelationalBranchIdentity,
        expected_root: Arc<RelationalBranchRoot>,
        prepared_root: Arc<RelationalBranchRoot>,
    ) -> Result<Self, super::RelationalRetentionAcquisitionDenial> {
        binding
            .acquire(
                RelationalRetentionObligationKind::Candidate,
                vec![expected_root, prepared_root],
                Some(identity),
            )
            .map(|guard| Self { guard })
    }

    pub(super) fn into_performed_settlement(
        mut self,
        current_root: Arc<RelationalBranchRoot>,
        owner: &Arc<super::owner::RelationalBranchRetentionOwnerInner>,
    ) -> RelationalPerformedSettlementObligation {
        self.guard
            .transfer_to_performed_settlement(current_root, owner);
        RelationalPerformedSettlementObligation { _guard: self.guard }
    }
}

#[derive(Debug)]
pub(crate) struct RelationalPerformedSettlementObligation {
    _guard: super::RelationalRetentionGuard,
}

impl RelationalPerformedSettlementObligation {
    pub(crate) fn record_interruption(&self, event: crate::runtime::RelationalInterruptionEvent) {
        self._guard.record_interruption(event);
    }
}

#[derive(Debug)]
pub(crate) struct RelationalExternalBasisRetentionObligation {
    guard: super::RelationalRetentionGuard,
}

impl RelationalExternalBasisRetentionObligation {
    pub(crate) fn acquire(
        binding: &super::RelationalBranchRetentionBinding,
        root: Arc<RelationalBranchRoot>,
    ) -> Result<Self, super::RelationalRetentionAcquisitionDenial> {
        binding
            .acquire(
                RelationalRetentionObligationKind::ExternalComponentBasis,
                vec![root],
                None,
            )
            .map(|guard| Self { guard })
    }

    pub(crate) fn owner_relationship(
        &self,
        binding: &super::RelationalBranchRetentionBinding,
    ) -> super::RelationalRetentionOwnerRelationship {
        self.guard.owner_relationship(binding)
    }

    pub(crate) fn release(mut self) -> super::RelationalBranchRetentionTerminalOutcome {
        self.guard.release_explicitly()
    }
}
