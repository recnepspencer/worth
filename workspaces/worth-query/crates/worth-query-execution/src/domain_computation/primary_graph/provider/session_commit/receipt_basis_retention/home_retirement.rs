//! Each original receipt-map slot retains its own Native retirement progress.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::change_source::{
    RelationalRetainedObservationRetirement,
    RelationalRetainedObservationRetirementProgress as NativeProgress,
    RetiredRelationalRetainedObservation,
};

pub(super) enum RetirableReceiptBasis {
    Serving(WorthQueryRetainedApplicationCommitBasis),
    Retiring(RelationalRetainedObservationRetirement),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Retired(RetiredRelationalRetainedObservation),
    LocalShareReleased,
}
impl RetirableReceiptBasis {
    pub(super) fn new(basis: WorthQueryRetainedApplicationCommitBasis) -> Self {
        Self::Serving(basis)
    }
    pub(super) fn retain_serving(&self) -> WorthQueryRetainedApplicationCommitBasis {
        match self {
            Self::Serving(basis) => basis.clone(),
            Self::Retiring(_) | Self::Retired(_) | Self::LocalShareReleased => {
                unreachable!("retiring provider exposes no receipt admission")
            }
        }
    }
    fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<Progress, Denial> {
        if matches!(self, Self::Retired(_) | Self::LocalShareReleased) {
            return Ok(Progress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        if matches!(self, Self::Serving(_)) {
            let Self::Serving(basis) = std::mem::replace(self, Self::LocalShareReleased) else {
                unreachable!()
            };
            if let Some(lease) = Arc::into_inner(basis.lease) {
                *self = Self::Retiring(lease.into_observation_retirement());
            }
            return Ok(Progress::Advanced);
        }
        let Self::Retiring(task) = self else {
            unreachable!("same original task")
        };
        match task.advance(request) {
            NativeProgress::Advanced => Ok(Progress::Advanced),
            NativeProgress::Busy => Err(Denial::Busy(Owner::ReceiptBases)),
            NativeProgress::Stopped(cause) => Err(Denial::Execution(cause)),
            NativeProgress::Refused(cause) => Err(Denial::NativeObservation(cause)),
            NativeProgress::Complete => {
                let Self::Retiring(task) = std::mem::replace(self, Self::LocalShareReleased) else {
                    unreachable!()
                };
                *self = Self::Retired(
                    task.into_retired()
                        .unwrap_or_else(|_| unreachable!("Native reported Complete")),
                );
                Ok(Progress::Complete)
            }
        }
    }
}

impl WorthQueryReceiptBasisRetentionStore {
    pub(in crate::domain_computation::primary_graph::provider) fn preflight_home_retirement(
        &self,
    ) -> Result<(), Denial> {
        if self.mandatory.is_empty() {
            Ok(())
        } else {
            Err(Denial::Unsupported(Owner::ReceiptBases))
        }
    }
    pub(in crate::domain_computation::primary_graph::provider) fn advance_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Progress, Denial> {
        use std::ops::Bound::{Excluded, Unbounded};
        self.preflight_home_retirement()?;
        let next = match self.retirement_cursor {
            Some(after) => self
                .by_commit
                .range_mut((Excluded(after), Unbounded))
                .next(),
            None => self.by_commit.iter_mut().next(),
        };
        let Some((commit, basis)) = next else {
            return Ok(Progress::Complete);
        };
        match basis.advance(request)? {
            Progress::Advanced => Ok(Progress::Advanced),
            Progress::Complete => {
                self.retirement_cursor = Some(*commit);
                Ok(Progress::Advanced)
            }
        }
    }
}
