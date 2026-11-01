//! Consume the issued external lease without running its ordinary release Drop.
use super::*;
use crate::history::retention::{
    RelationalRetentionRetirementCustody, RelationalRetentionRetirementError,
};
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub(crate) struct RelationalExternalLeaseRetirement {
    lease: RelationalBranchRetentionLease,
    custody: RelationalRetentionRetirementCustody,
    observation: Option<RelationalBranchObservation>,
    released: bool,
}

impl RelationalBranchRetentionLease {
    pub(crate) fn into_retirement(self) -> RelationalExternalLeaseRetirement {
        RelationalExternalLeaseRetirement {
            lease: self,
            custody: RelationalRetentionRetirementCustody::default(),
            observation: None,
            released: false,
        }
    }
}

impl RelationalExternalLeaseRetirement {
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRetentionRetirementError> {
        if self.released {
            return Ok(());
        }
        self.lease
            .obligation
            .as_mut()
            .ok_or(RelationalRetentionRetirementError::RegistrationMismatch)?
            .try_retire_retaining(&mut self.custody, request)?;
        self.lease.terminal_accounting.record_explicit_release();
        self.observation = self.lease.observation.take();
        self.released = true;
        Ok(())
    }

    pub(crate) fn take_retired_observation(&mut self) -> Option<RelationalBranchObservation> {
        if self.released {
            self.observation.take()
        } else {
            None
        }
    }
}
