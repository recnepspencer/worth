//! The two original shares of an externally retained observation, with no read door.
use super::*;
use crate::branch::RelationalBasisRegistryRetirementCustody;
use crate::history::retention::{
    RelationalRetentionRetirementCustody, RelationalRetentionRetirementError,
};
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub(crate) struct RelationalRetainedObservationShares {
    first: Option<RelationalBranchObservation>,
    second: Option<RelationalBranchObservation>,
    inner: Option<AdmittedRelationalBranchBasisInner>,
    observation_released: bool,
    registry_released: bool,
    observation_custody: RelationalRetentionRetirementCustody,
    registry_custody: RelationalBasisRegistryRetirementCustody,
}

impl RelationalRetainedObservationShares {
    pub(crate) fn new(
        first: RelationalBranchObservation,
        second: RelationalBranchObservation,
    ) -> Self {
        Self {
            first: Some(first),
            second: Some(second),
            inner: None,
            observation_released: false,
            registry_released: false,
            observation_custody: RelationalRetentionRetirementCustody::default(),
            registry_custody: RelationalBasisRegistryRetirementCustody::default(),
        }
    }

    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, RelationalRetentionRetirementError> {
        if let Some(observation) = self.first.take() {
            self.inner = Arc::into_inner(observation.inner);
            return Ok(false);
        }
        if let Some(observation) = self.second.take() {
            self.inner = Arc::into_inner(observation.inner);
            return Ok(false);
        }
        let Some(inner) = self.inner.as_mut() else {
            return Ok(true);
        };
        if !self.observation_released {
            inner
                .retention
                .try_retire_retaining(&mut self.observation_custody, request)?;
            self.observation_released = true;
            return Ok(false);
        }
        if !self.registry_released {
            if let Some(lease) = inner.registry_lease.get_mut() {
                lease.try_retire_retaining(&mut self.registry_custody, request)?;
            }
            self.registry_released = true;
        }
        Ok(true)
    }
}
