use worth_query_execution_contracts::facade::relational_execution_kind;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

use super::super::lease::validation::{open_denial, subscription_capacity_denial};
use super::super::outcome::{
    WorthQueryApplicationLiveOpenDenial, WorthQueryApplicationLiveOpenDenialKind,
};
use super::footprint::FootprintStop;
use super::{
    WorthQueryApplicationWholeResultLiveLease, WorthQueryApplicationWholeResultLiveOutcome,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::application_query::{
    authorized_read::WorthQueryAuthorizedApplicationReadDenial,
    read_execution::WorthQueryApplicationReadExecutionDenialKind as ReadKind,
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationOneShotDenialKind as OneShotKind,
    WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind as AdmissionKind,
};
use crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::WorthQueryProviderSessionControlStopKind as ControlStop;

pub(super) fn map_open_live_admission_denial(
    denial: WorthQueryLiveAdmissionDenial,
    subject: &str,
) -> WorthQueryApplicationLiveOpenDenial {
    match denial {
        WorthQueryLiveAdmissionDenial::Execution(cause) => {
            open_denial(open_relational_kind(cause), subject)
        }
        WorthQueryLiveAdmissionDenial::Unavailable(_) => subscription_capacity_denial(subject),
    }
}

pub(super) fn map_open_branch_admission_denial(
    cause: crate::basis::WorthQueryProductBranchAdmissionDenial,
    subject: &str,
) -> WorthQueryApplicationLiveOpenDenial {
    let kind = match cause {
        crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(Ok(kind)) => {
            WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::ExecutionRequest(
                kind,
            ))
        }
        crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(Err(
            ControlStop::Cancelled,
        )) => WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::Cancelled),
        crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(Err(
            ControlStop::TimedOut,
        )) => WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::DeadlineExceeded),
        crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_) => {
            WorthQueryApplicationLiveOpenDenialKind::ProviderVersionUnavailable
        }
    };
    open_denial(kind, subject)
}

pub(super) fn map_open_one_shot_denial(
    denial: WorthQueryApplicationOneShotDenial,
    subject: &str,
) -> WorthQueryApplicationLiveOpenDenial {
    let kind = match denial.kind() {
        OneShotKind::Cancelled => {
            WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::Cancelled)
        }
        OneShotKind::DeadlineExceeded => {
            WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::DeadlineExceeded)
        }
        OneShotKind::ExecutionRequest(kind) => WorthQueryApplicationLiveOpenDenialKind::Admission(
            AdmissionKind::ExecutionRequest(kind),
        ),
        OneShotKind::WorkLimitExceeded => open_relational_kind(Cause::WorkExhausted),
        OneShotKind::StalePrincipal => {
            WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::StalePrincipal)
        }
        OneShotKind::BasisReleaseFailed => {
            WorthQueryApplicationLiveOpenDenialKind::BasisReleaseFailed
        }
        OneShotKind::Content(cause) => WorthQueryApplicationLiveOpenDenialKind::Content(cause),
        _ => WorthQueryApplicationLiveOpenDenialKind::ScopeIdentityUnavailable,
    };
    open_denial(kind, subject)
}

pub(super) fn map_open_footprint_stop(
    stop: FootprintStop,
    subject: &str,
) -> WorthQueryApplicationLiveOpenDenial {
    let kind = match stop {
        FootprintStop::Cancelled => {
            WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::Cancelled)
        }
        FootprintStop::DeadlineExceeded => {
            WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::DeadlineExceeded)
        }
        FootprintStop::Execution(cause) => open_relational_kind(cause),
        FootprintStop::Unavailable => {
            WorthQueryApplicationLiveOpenDenialKind::ScopeIdentityUnavailable
        }
    };
    open_denial(kind, subject)
}

fn open_relational_kind(cause: Cause) -> WorthQueryApplicationLiveOpenDenialKind {
    match relational_execution_kind(cause, None) {
        Ok(kind) => WorthQueryApplicationLiveOpenDenialKind::Admission(
            AdmissionKind::ExecutionRequest(kind),
        ),
        Err(ControlStop::Cancelled) => {
            WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::Cancelled)
        }
        Err(ControlStop::TimedOut) => {
            WorthQueryApplicationLiveOpenDenialKind::Admission(AdmissionKind::DeadlineExceeded)
        }
    }
}

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
    pub(super) fn handle_basis_admission_denial(
        &mut self,
        cause: crate::basis::WorthQueryProductBranchAdmissionDenial,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        match cause {
            crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(Ok(kind)) => {
                self.execution_denied_outcome(kind)
            }
            crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(Err(
                ControlStop::Cancelled,
            )) => self.cancelled_outcome(),
            crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(Err(
                ControlStop::TimedOut,
            )) => self.deadline_exceeded_outcome(),
            crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_) => {
                self.terminate_outcome(WorthQueryApplicationWholeResultLiveOutcome::Unavailable)
            }
        }
    }

    pub(super) fn handle_relational_cause(
        &mut self,
        cause: Cause,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        match relational_execution_kind(cause, None) {
            Ok(kind) => self.execution_denied_outcome(kind),
            Err(ControlStop::Cancelled) => self.cancelled_outcome(),
            Err(ControlStop::TimedOut) => self.deadline_exceeded_outcome(),
        }
    }

    pub(super) fn handle_live_admission_denial(
        &mut self,
        denial: WorthQueryLiveAdmissionDenial,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        match denial {
            WorthQueryLiveAdmissionDenial::Execution(cause) => self.handle_relational_cause(cause),
            WorthQueryLiveAdmissionDenial::Unavailable(_) => {
                self.terminate_outcome(WorthQueryApplicationWholeResultLiveOutcome::Unavailable)
            }
        }
    }

    pub(super) fn handle_footprint_stop(
        &mut self,
        stop: FootprintStop,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        match stop {
            FootprintStop::Cancelled => self.cancelled_outcome(),
            FootprintStop::DeadlineExceeded => self.deadline_exceeded_outcome(),
            FootprintStop::Execution(cause) => self.handle_relational_cause(cause),
            FootprintStop::Unavailable => {
                self.terminate_outcome(WorthQueryApplicationWholeResultLiveOutcome::Unavailable)
            }
        }
    }

    pub(super) fn handle_admission_denial(
        &mut self,
        denial: WorthQueryApplicationQueryAdmissionDenial,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        use WorthQueryApplicationWholeResultLiveOutcome as Outcome;
        match denial.kind() {
            AdmissionKind::ExecutionRequest(kind) => self.execution_denied_outcome(kind),
            AdmissionKind::Cancelled => self.cancelled_outcome(),
            AdmissionKind::DeadlineExceeded => self.deadline_exceeded_outcome(),
            AdmissionKind::StalePrincipal | AdmissionKind::ForeignPrincipal => {
                self.terminate_outcome(Outcome::StalePrincipal)
            }
            AdmissionKind::StaleScope
            | AdmissionKind::ForeignScope
            | AdmissionKind::ScopeTypeMismatch
            | AdmissionKind::HistoricalSourceNotLive { .. }
            | AdmissionKind::HistoricalSourceKindMismatch { .. }
            | AdmissionKind::HistoricalSourceOutsideDomain { .. }
            | AdmissionKind::HistoricalSourceOwnershipUnavailable { .. } => {
                self.terminate_outcome(Outcome::StaleScope)
            }
            AdmissionKind::Authorization(_) => {
                let auth = denial.into_authorization_denial().unwrap_or_else(|| {
                    WorthQueryOperationAuthorizationDenial::inconsistent(self.query.name())
                });
                self.handle_authorization_denial(auth)
            }
            AdmissionKind::DisclosureGovernanceRequired
            | AdmissionKind::DisclosureAuthorizationMismatch
            | AdmissionKind::InternalComputationDenied => {
                let auth = WorthQueryOperationAuthorizationDenial::new(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::InconsistentDecision,
                    ),
                    self.query.name(),
                );
                self.handle_authorization_denial(auth)
            }
            _ => self.terminate_outcome(Outcome::Unavailable),
        }
    }

    pub(super) fn handle_read_denial(
        &mut self,
        denial: WorthQueryAuthorizedApplicationReadDenial,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        use WorthQueryApplicationWholeResultLiveOutcome as Outcome;
        match denial {
            WorthQueryAuthorizedApplicationReadDenial::StalePrincipal => {
                self.terminate_outcome(Outcome::StalePrincipal)
            }
            WorthQueryAuthorizedApplicationReadDenial::ScopeFreshness(_) => {
                self.terminate_outcome(Outcome::StaleScope)
            }
            WorthQueryAuthorizedApplicationReadDenial::Authorization(auth) => {
                self.handle_authorization_denial(auth)
            }
            WorthQueryAuthorizedApplicationReadDenial::Read(read) => match read.kind() {
                ReadKind::ExecutionRequest(kind) => self.execution_denied_outcome(kind),
                ReadKind::Cancelled => self.cancelled_outcome(),
                ReadKind::DeadlineExceeded => self.deadline_exceeded_outcome(),
                ReadKind::ResultLimitExceeded => {
                    self.terminate_outcome(Outcome::ResultLimitExceeded)
                }
                ReadKind::WorkLimitExceeded => self.handle_relational_cause(Cause::WorkExhausted),
                _ => self.terminate_outcome(Outcome::Unavailable),
            },
            WorthQueryAuthorizedApplicationReadDenial::Session => {
                self.terminate_outcome(Outcome::Unavailable)
            }
        }
    }

    pub(super) fn handle_one_shot_denial(
        &mut self,
        denial: WorthQueryApplicationOneShotDenial,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        use WorthQueryApplicationWholeResultLiveOutcome as Outcome;
        match denial.kind() {
            OneShotKind::ExecutionRequest(kind) => self.execution_denied_outcome(kind),
            OneShotKind::Cancelled => self.cancelled_outcome(),
            OneShotKind::DeadlineExceeded => self.deadline_exceeded_outcome(),
            OneShotKind::StalePrincipal => self.terminate_outcome(Outcome::StalePrincipal),
            OneShotKind::ResultLimitExceeded => {
                self.terminate_outcome(Outcome::ResultLimitExceeded)
            }
            OneShotKind::Projection(kind) => {
                self.terminate_outcome(Outcome::ProjectionDenied(kind))
            }
            OneShotKind::WorkLimitExceeded => self.handle_relational_cause(Cause::WorkExhausted),
            _ => self.terminate_outcome(Outcome::Unavailable),
        }
    }

    fn handle_authorization_denial(
        &mut self,
        denial: WorthQueryOperationAuthorizationDenial,
    ) -> WorthQueryApplicationWholeResultLiveOutcome<Query, QueryResult> {
        match denial.kind() {
            WorthQueryOperationAuthorizationDenialKind::Stopped(Ok(kind)) => {
                self.execution_denied_outcome(kind)
            }
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(ControlStop::Cancelled)) => {
                self.cancelled_outcome()
            }
            WorthQueryOperationAuthorizationDenialKind::Stopped(Err(ControlStop::TimedOut)) => {
                self.deadline_exceeded_outcome()
            }
            WorthQueryOperationAuthorizationDenialKind::Refused(_) => self.terminate_outcome(
                WorthQueryApplicationWholeResultLiveOutcome::AuthorizationDenied(Box::new(denial)),
            ),
        }
    }
}
