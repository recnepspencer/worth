//! Remove one issued obligation while keeping its original allocations alive.
use super::*;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use tests::assert_external_cut_and_lane_busy;

#[derive(Debug, Default)]
pub(crate) struct RelationalRetentionRetirementCustody {
    owner: Option<Arc<RelationalBranchRetentionOwnerInner>>,
    removed: Option<ExecutionOrderedMapCell<u64, RelationalRetentionObligationRecord>>,
    removed_counts: [Option<ExecutionOrderedMapCell<u64, usize>>; 2],
}

#[derive(Debug)]
pub(crate) enum RelationalRetentionRetirementError {
    Busy,
    Stopped(WorkCeilingDenial),
    OwnerUnavailable,
    RegistrationMismatch,
    AccountingMismatch,
}

impl RelationalRetentionRetirementCustody {
    fn retain_owner(
        &mut self,
        guard: &RelationalRetentionGuard,
    ) -> Result<(), RelationalRetentionRetirementError> {
        if self.owner.is_none() {
            self.owner = Some(
                guard
                    .owner
                    .upgrade()
                    .ok_or(RelationalRetentionRetirementError::OwnerUnavailable)?,
            );
        }
        Ok(())
    }
}

impl RelationalRetentionGuard {
    pub(crate) fn try_retire_external_retaining(
        &mut self,
        custody: &mut RelationalRetentionRetirementCustody,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRetentionRetirementError> {
        if self.terminal {
            return Ok(());
        }
        custody.retain_owner(self)?;
        let owner = custody.owner.as_ref().expect(/* Proof: retain_owner installs the actual upgraded owner or returns before this point. */ "retirement retains its owner");
        let _cut = match owner.external_cut.try_lock() {
            Ok(value) => value,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err(RelationalRetentionRetirementError::Busy)
            }
        };
        self.remove_retaining(
            owner,
            &mut custody.removed,
            &mut custody.removed_counts,
            RelationalRetentionObligationKind::ExternalComponentBasis,
            request,
        )
    }

    pub(crate) fn try_retire_observation_retaining(
        &mut self,
        custody: &mut RelationalRetentionRetirementCustody,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRetentionRetirementError> {
        if self.terminal {
            return Ok(());
        }
        custody.retain_owner(self)?;
        self.remove_retaining(custody.owner.as_ref().expect(/* Proof: retain_owner installs the actual upgraded owner or returns before this point. */ "retirement retains its owner"), &mut custody.removed, &mut custody.removed_counts, RelationalRetentionObligationKind::Observation, request)
    }

    fn remove_retaining(
        &mut self,
        owner: &Arc<RelationalBranchRetentionOwnerInner>,
        removed: &mut Option<ExecutionOrderedMapCell<u64, RelationalRetentionObligationRecord>>,
        removed_counts: &mut [Option<ExecutionOrderedMapCell<u64, usize>>; 2],
        expected_kind: RelationalRetentionObligationKind,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRetentionRetirementError> {
        let mut state = match self.lane.state.try_lock() {
            Ok(value) => value,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err(RelationalRetentionRetirementError::Busy)
            }
        };
        let record = state
            .obligations
            .get(&self.obligation_id)
            .ok_or(RelationalRetentionRetirementError::RegistrationMismatch)?;
        let ids = record.root_ids();
        if record.kind() != expected_kind
            || record.active_operation
            || self.roots.len() != 1
            || ids.len() != 1
            || ids[0] != self.roots[0].id()
            || !Weak::ptr_eq(&self.owner, &Arc::downgrade(owner))
            || removed.is_some()
        {
            return Err(RelationalRetentionRetirementError::RegistrationMismatch);
        }
        if state.root_obligations.get(&ids[0]).copied().unwrap_or(0) == 0
            || !super::super::owner::has_live_obligation_capacity(owner)
        {
            return Err(RelationalRetentionRetirementError::AccountingMismatch);
        }
        request
            .consult()
            .map_err(RelationalRetentionRetirementError::Stopped)?;
        // The non-Clone guard owns this exact row. No fallible work follows removal.
        let record = state.obligations.remove_owned_retaining_unique(&self.obligation_id).expect("private obligation storage is never cloned").expect(/* Proof: The exact registered row was validated under this same lane guard and no intervening mutation occurs. */ "validated obligation remains registered");
        let value = record.value();
        *removed_counts = state.release_roots(value.root_ids());
        record_release(&mut state.counters, value.kind());
        owner.ordinary_counters.record_release(value.kind());
        release_live_obligation_capacity(owner);
        *removed = Some(record);
        self.terminal = true;
        Ok(())
    }
}
