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
    pub(crate) fn try_retire_retaining(
        &mut self,
        custody: &mut super::RelationalRetentionRetirementCustody,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), super::RelationalRetentionRetirementError> {
        self._guard
            .try_retire_observation_retaining(custody, request)
    }
    pub(crate) fn try_retire(&mut self) -> bool {
        self._guard.try_retire_observation()
    }
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
    pub(super) fn try_retire_observation(&mut self) -> bool {
        self._obligation.try_retire()
    }
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
    guard: Option<super::RelationalRetentionGuard>,
}

impl RelationalExternalBasisRetentionObligation {
    pub(crate) fn try_retire_retaining(
        &mut self,
        custody: &mut super::RelationalRetentionRetirementCustody,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), super::RelationalRetentionRetirementError> {
        self.guard
            .as_mut()
            .ok_or(super::RelationalRetentionRetirementError::RegistrationMismatch)?
            .try_retire_external_retaining(custody, request)
    }
    pub(crate) fn acquire(
        binding: &super::RelationalBranchRetentionBinding,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> Result<Self, super::RelationalRetentionAcquisitionDenial> {
        let owner = binding
            .owner
            .upgrade()
            .ok_or(super::RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        let _cut = owner
            .external_cut
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        binding
            .acquire_component(basis)
            .map(|guard| Self { guard: Some(guard) })
    }

    pub(crate) fn owner_relationship(
        &self,
        binding: &super::RelationalBranchRetentionBinding,
    ) -> super::RelationalRetentionOwnerRelationship {
        self.guard
            .as_ref()
            .map(|guard| guard.owner_relationship(binding))
            .unwrap_or(super::RelationalRetentionOwnerRelationship::OwnerUnavailable)
    }

    pub(crate) fn release(mut self) -> super::RelationalBranchRetentionTerminalOutcome {
        self.release_guard()
    }
}

impl RelationalExternalBasisRetentionObligation {
    fn release_guard(&mut self) -> super::RelationalBranchRetentionTerminalOutcome {
        let Some(mut guard) = self.guard.take() else {
            return super::RelationalBranchRetentionTerminalOutcome::Released;
        };
        let owner = guard.owner.upgrade();
        let _cut = owner.as_ref().map(|owner| {
            owner
                .external_cut
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
        });
        guard.release_explicitly()
    }
}
impl Drop for RelationalExternalBasisRetentionObligation {
    fn drop(&mut self) {
        self.release_guard();
    }
}
