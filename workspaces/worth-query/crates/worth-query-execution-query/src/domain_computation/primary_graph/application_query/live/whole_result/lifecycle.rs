use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_runtime_bridge::facade::BridgeExecutionBasisTerminalDisposition;

use super::super::outcome::WorthQueryApplicationLiveCloseOutcome;
use super::{
    WorthQueryApplicationWholeResultLiveLease, WorthQueryApplicationWholeResultLiveOutcome,
};

impl<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryApplicationWholeResultLiveLease<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    pub fn observe_interruption(
        &mut self,
        request: &WorthQueryRequestScope,
    ) -> Option<WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult>> {
        if self.termination_failed || self.basis.is_none() {
            return Some(WorthQueryApplicationWholeResultLiveOutcome::Closed);
        }
        let interruption = request.interruption()?;
        self.controls.replace_request(request.clone());
        Some(match interruption {
            WorthQueryRequestInterruption::Cancelled => self.cancelled_outcome(),
            WorthQueryRequestInterruption::DeadlineExceeded => self.deadline_exceeded_outcome(),
        })
    }

    pub fn terminate_stale_principal(
        &mut self,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        self.terminate_outcome(WorthQueryApplicationWholeResultLiveOutcome::StalePrincipal)
    }

    pub fn end_with_read_stop(
        &mut self,
        cause: Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        match cause {
            Ok(kind) => self.execution_denied_outcome(kind),
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled) => {
                self.cancelled_outcome()
            }
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut) => {
                self.deadline_exceeded_outcome()
            }
        }
    }

    pub fn close(mut self) -> WorthQueryApplicationLiveCloseOutcome {
        if self.terminate(BridgeExecutionBasisTerminalDisposition::Completed) {
            self.read_completion.take().map_or(
                WorthQueryApplicationLiveCloseOutcome::Unavailable,
                WorthQueryApplicationLiveCloseOutcome::Completed,
            )
        } else {
            WorthQueryApplicationLiveCloseOutcome::Unavailable
        }
    }

    pub(super) fn terminate(
        &mut self,
        disposition: BridgeExecutionBasisTerminalDisposition,
    ) -> bool {
        let Some(basis) = self.basis.take() else {
            return true;
        };
        let crate::domain_computation::managed_run::WorthQueryManagedLowerExecutionBasis {
            bridge,
            relational,
            product_observation,
        } = basis;
        match bridge.finalize(disposition) {
            Ok(_) => true,
            Err(failure) => {
                self.basis = Some(
                    crate::domain_computation::managed_run::WorthQueryManagedLowerExecutionBasis {
                        bridge: failure.into_basis(),
                        relational,
                        product_observation,
                    },
                );
                false
            }
        }
    }

    pub(super) fn terminate_stopped_read(&mut self) {
        if !self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled) {
            self.termination_failed = true;
        }
    }

    pub(super) fn terminate_outcome(
        &mut self,
        outcome: WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult>,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        if self.terminate(BridgeExecutionBasisTerminalDisposition::Cancelled) {
            outcome
        } else {
            WorthQueryApplicationWholeResultLiveOutcome::Unavailable
        }
    }

    pub(super) fn execution_denied_outcome(
        &mut self,
        kind: crate::domain_computation::WorthQueryProviderSessionDenialKind,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        self.terminate_stopped_read();
        WorthQueryApplicationWholeResultLiveOutcome::ExecutionRequest(kind)
    }

    pub(super) fn cancelled_outcome(
        &mut self,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        self.terminate_stopped_read();
        WorthQueryApplicationWholeResultLiveOutcome::Cancelled
    }

    pub(super) fn deadline_exceeded_outcome(
        &mut self,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        self.terminate_stopped_read();
        WorthQueryApplicationWholeResultLiveOutcome::DeadlineExceeded
    }
}

impl<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope> Drop
    for WorthQueryApplicationWholeResultLiveLease<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    fn drop(&mut self) {
        let _ = self.terminate(BridgeExecutionBasisTerminalDisposition::Abandoned);
    }
}
