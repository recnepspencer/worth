//! Each original receipt-map slot retains its own Native retirement progress.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchCloseCustody;
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
    /// A dispatch-owned basis keeps its commit readable for the dispatch
    /// that still owes a terminal. Close may release it only when that
    /// dispatch's custody is durable: the outbox row is in the commit and
    /// every attempt and observation is in `query.dispatch`.
    pub(in crate::domain_computation::primary_graph::provider) fn preflight_home_retirement(
        &self,
        dispatch_custody: &QueryDispatchCloseCustody,
    ) -> Result<(), Denial> {
        match dispatch_custody {
            QueryDispatchCloseCustody::Durable(_admitted_prefix) => Ok(()),
            QueryDispatchCloseCustody::ProcessLocal if self.mandatory.is_empty() => Ok(()),
            QueryDispatchCloseCustody::ProcessLocal => {
                Err(Denial::Unsupported(Owner::ReceiptBases))
            }
        }
    }
    pub(in crate::domain_computation::primary_graph::provider) fn advance_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        dispatch_custody: &QueryDispatchCloseCustody,
    ) -> Result<Progress, Denial> {
        use std::ops::Bound::{Excluded, Unbounded};
        self.preflight_home_retirement(dispatch_custody)?;
        // One commit opens one basis, ordinary or dispatch-owned, so one
        // cursor walks both maps in commit order.
        let after = match self.retirement_cursor {
            Some(after) => Excluded(after),
            None => Unbounded,
        };
        let ordinary = self
            .by_commit
            .range_mut((after, Unbounded))
            .next()
            .map(|(commit, basis)| (*commit, basis));
        let owned = self
            .mandatory
            .range_mut((after, Unbounded))
            .next()
            .map(|(commit, (basis, _claims))| (*commit, basis));
        let next = match (ordinary, owned) {
            (Some(ordinary), Some(owned)) if owned.0 < ordinary.0 => Some(owned),
            (Some(ordinary), Some(_) | None) => Some(ordinary),
            (None, owned) => owned,
        };
        let Some((commit, basis)) = next else {
            return Ok(Progress::Complete);
        };
        match basis.advance(request)? {
            Progress::Advanced => Ok(Progress::Advanced),
            Progress::Complete => {
                self.retirement_cursor = Some(commit);
                Ok(Progress::Advanced)
            }
        }
    }
}
