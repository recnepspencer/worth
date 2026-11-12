//! Original admitted-basis custody retires through Native's observation owner.
use super::AdmittedRelationalBranchBasis;
use crate::mvcc::RelationalObservationRetirement;
use crate::runtime::RelationalRuntime;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

#[derive(Debug)]
pub enum RelationalBranchBasisRetirementDenial {
    ForeignOwner,
    Busy,
    Execution(WorkCeilingDenial),
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RelationalBranchBasisRetirementProgress {
    Advanced,
    Complete,
}

/// Consumes one actual basis share; it exposes no continuing read capability.
#[derive(Debug)]
#[must_use]
pub struct RelationalBranchBasisRetirement {
    original: RelationalObservationRetirement,
    owner: u64,
    complete: bool,
}
/// Retired custody cannot reopen an original read.
/// ```compile_fail
/// use worth_relational::facade::branch::RetiredRelationalBranchBasis;
/// fn cannot_read(value: RetiredRelationalBranchBasis) { value.observation(); }
/// ```
#[derive(Debug)]
#[must_use]
pub struct RetiredRelationalBranchBasis {
    _original: RelationalBranchBasisRetirement,
}

impl AdmittedRelationalBranchBasis {
    pub fn into_retirement(self) -> RelationalBranchBasisRetirement {
        let owner = self.identity().runtime_instance_id();
        RelationalBranchBasisRetirement {
            original: RelationalObservationRetirement::from_original_basis(self),
            owner,
            complete: false,
        }
    }
}
impl RelationalBranchBasisRetirement {
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        owner: &RelationalRuntime,
    ) -> Result<RelationalBranchBasisRetirementProgress, RelationalBranchBasisRetirementDenial>
    {
        use RelationalBranchBasisRetirementDenial as Denial;
        use RelationalBranchBasisRetirementProgress as Progress;
        if owner.runtime_instance_id() != self.owner {
            return Err(Denial::ForeignOwner);
        }
        if self.complete {
            return Ok(Progress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        self.complete = self
            .original
            .advance_retaining_allocation()
            .map_err(|()| Denial::Busy)?;
        Ok(if self.complete {
            Progress::Complete
        } else {
            Progress::Advanced
        })
    }
    pub fn into_retired(self) -> Result<RetiredRelationalBranchBasis, Self> {
        if self.complete {
            Ok(RetiredRelationalBranchBasis { _original: self })
        } else {
            Err(self)
        }
    }
}

impl RetiredRelationalBranchBasis {
    /// Explicitly unbudgeted destruction of the retained original payload.
    pub fn reclaim(self) {
        drop(self);
    }
}
#[cfg(test)]
#[path = "retirement_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
