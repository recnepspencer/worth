//! Request-controlled consumption of the original external and observation custody.
use super::RelationalRetainedObservation;
use crate::history::retention::{
    RelationalExternalLeaseRetirement, RelationalRetentionRetirementError,
};
use crate::mvcc::{RelationalBranchObservation, RelationalRetainedObservationShares};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRetainedObservationRetirementDenial {
    OwnerUnavailable,
    RegistrationMismatch,
    AccountingMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRetainedObservationRetirementProgress {
    Advanced,
    Busy,
    Stopped(WorkCeilingDenial),
    Refused(RelationalRetainedObservationRetirementDenial),
    Complete,
}

/// Consuming custody of one issued retention. It exposes no observation or
/// service access. Abandoning it runs ordinary, explicitly unbudgeted cleanup.
#[derive(Debug)]
#[must_use]
pub struct RelationalRetainedObservationRetirement {
    observation: Option<RelationalBranchObservation>,
    external: RelationalExternalLeaseRetirement,
    external_released: bool,
    shares: Option<RelationalRetainedObservationShares>,
    complete: bool,
}

/// Original allocations retained after their local registrations are retired.
/// Independent aliases keep their own read custody. This is not Native close.
#[derive(Debug)]
#[must_use]
pub struct RetiredRelationalRetainedObservation {
    _custody: RelationalRetainedObservationRetirement,
}

impl RelationalRetainedObservation {
    /// Move the original payload into retirement without allocation or release.
    pub fn into_retirement(self) -> RelationalRetainedObservationRetirement {
        RelationalRetainedObservationRetirement {
            observation: Some(self.observation),
            external: self.retention.into_retirement(),
            external_released: false,
            shares: None,
            complete: false,
        }
    }
}

impl RelationalRetainedObservationRetirement {
    pub(crate) fn from_basis_and_lease(
        basis: crate::branch::AdmittedRelationalBranchBasis,
        lease: crate::history::retention::RelationalBranchRetentionLease,
    ) -> Self {
        assert_eq!(basis.descriptor(), lease.descriptor());
        Self {
            observation: Some(basis.into_observation()),
            external: lease.into_retirement(),
            external_released: false,
            shares: None,
            complete: false,
        }
    }
    pub(crate) fn from_retention_lease(
        lease: crate::history::retention::RelationalBranchRetentionLease,
    ) -> Self {
        Self {
            observation: None,
            external: lease.into_retirement(),
            external_released: false,
            shares: None,
            complete: false,
        }
    }
    /// Advance one original stage without waiting. Every live step borrows the
    /// original request; completed tasks do not poll or repeat effects.
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> RelationalRetainedObservationRetirementProgress {
        if self.complete {
            return RelationalRetainedObservationRetirementProgress::Complete;
        }
        if let Err(cause) = request.consult() {
            return RelationalRetainedObservationRetirementProgress::Stopped(cause);
        }
        match self.advance_original_stage(request) {
            Ok(false) => RelationalRetainedObservationRetirementProgress::Advanced,
            Ok(true) => {
                self.complete = true;
                RelationalRetainedObservationRetirementProgress::Complete
            }
            Err(error) => error.into(),
        }
    }

    /// Refuse to turn unfinished obligations into retired custody.
    pub fn into_retired(self) -> Result<RetiredRelationalRetainedObservation, Self> {
        if self.complete {
            Ok(RetiredRelationalRetainedObservation { _custody: self })
        } else {
            Err(self)
        }
    }

    fn advance_original_stage(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, RelationalRetentionRetirementError> {
        if !self.external_released {
            self.external.advance(request)?;
            self.external_released = true;
            return Ok(false);
        }
        if self.shares.is_none() {
            let second = self.external.take_retired_observation().expect(/* Proof: The authentic external lease owns its second share, and external_released is set only after successful retirement. */ "retired external lease retains its observation");
            self.shares = Some(match self.observation.take() {
                Some(first) => RelationalRetainedObservationShares::new(first, second),
                None => RelationalRetainedObservationShares::single(second),
            });
            return Ok(false);
        }
        self.shares.as_mut().expect(/* Proof: The preceding branch installs shares and returns; every later step retains that same task. */ "original share retirement is installed").advance(request)
    }
}

impl RetiredRelationalRetainedObservation {
    /// Destroy retained data only outside a request-controlled operation.
    pub fn reclaim(self) {
        drop(self);
    }
}

impl From<RelationalRetentionRetirementError> for RelationalRetainedObservationRetirementProgress {
    fn from(error: RelationalRetentionRetirementError) -> Self {
        use RelationalRetainedObservationRetirementDenial as Denial;
        match error {
            RelationalRetentionRetirementError::Busy => Self::Busy,
            RelationalRetentionRetirementError::Stopped(cause) => Self::Stopped(cause),
            RelationalRetentionRetirementError::OwnerUnavailable => {
                Self::Refused(Denial::OwnerUnavailable)
            }
            RelationalRetentionRetirementError::RegistrationMismatch => {
                Self::Refused(Denial::RegistrationMismatch)
            }
            RelationalRetentionRetirementError::AccountingMismatch => {
                Self::Refused(Denial::AccountingMismatch)
            }
        }
    }
}
