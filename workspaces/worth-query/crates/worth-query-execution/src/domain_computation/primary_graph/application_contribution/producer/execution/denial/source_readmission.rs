//! Request rejection remains local to the caller through both query phases.

use super::request_execution::kind as request_execution_kind;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind as DemandKind;
use super::{
    denial, failed, request_admission_denied, request_admission_rejected, request_authority_stop,
    ProducerExecutionStop,
};
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationOneShotDenialKind as ExecutionKind,
    WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind as AdmissionKind,
};

pub fn query_admission_denied(
    subject: &str,
    error: WorthQueryApplicationQueryAdmissionDenial,
) -> ProducerExecutionStop {
    if admission_authority_rejection(error.kind()) {
        return match error.kind() {
            AdmissionKind::Authorization(kind) => {
                request_admission_rejected(request_authority_stop(subject, kind))
            }
            _ => request_admission_denied(subject, error),
        };
    }
    match error.kind() {
        AdmissionKind::ExecutionRequest(cause) => ProducerExecutionStop::ExecutionStopped(denial(
            request_execution_kind(cause),
            format!("{subject}: {error:?}"),
        )),
        AdmissionKind::Cancelled => {
            request_admission_rejected(denial(DemandKind::Cancelled, subject.to_owned()))
        }
        AdmissionKind::DeadlineExceeded => {
            request_admission_rejected(denial(DemandKind::TimedOut, subject.to_owned()))
        }
        other => {
            let kind = match other {
                AdmissionKind::WorkLimitExceeded => DemandKind::WorkBudgetExceeded,
                AdmissionKind::ReadmissionPreparationMemoryExhausted => {
                    DemandKind::RetentionBudgetExceeded
                }
                _ => DemandKind::ProducerUnavailable,
            };
            ProducerExecutionStop::ExecutionStopped(denial(kind, format!("{subject}: {error:?}")))
        }
    }
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn query_execution_denied(
    subject: &str,
    error: WorthQueryApplicationOneShotDenial,
) -> ProducerExecutionStop {
    if execution_authority_rejection(error.kind()) {
        return match error.kind() {
            ExecutionKind::Authorization(kind) => {
                request_admission_rejected(request_authority_stop(subject, kind))
            }
            _ => request_admission_denied(subject, error),
        };
    }
    match error.kind() {
        ExecutionKind::ExecutionRequest(cause) => ProducerExecutionStop::ExecutionStopped(denial(
            request_execution_kind(cause),
            format!("{subject}: {error:?}"),
        )),
        ExecutionKind::Cancelled => {
            request_admission_rejected(denial(DemandKind::Cancelled, subject.to_owned()))
        }
        ExecutionKind::DeadlineExceeded => {
            request_admission_rejected(denial(DemandKind::TimedOut, subject.to_owned()))
        }
        _ => ProducerExecutionStop::ExecutionStopped(failed(subject, error)),
    }
}

/// Selected Source disclosure preserves the original admitted read or managed cause.
pub fn selected_source_read_denied(
    subject: &'static str,
    stop: crate::domain_computation::primary_graph::application_query::SourceQueryObservationStop,
) -> ProducerExecutionStop {
    use crate::domain_computation::primary_graph::application_query::{
        SourceQueryObservationStop as Stop, WorthQueryAdmittedOneShotStop as Read,
    };
    match stop {
        Stop::CurrentControl(stop) => request_admission_rejected(stop),
        Stop::Read(Read::Admission(stop)) => {
            super::super::source_readmission::ready_resource_denial(subject, stop)
        }
        Stop::Read(Read::WorkCounterOverflow | Read::WorkAccountingMismatch) => {
            ProducerExecutionStop::ExecutionStopped(denial(DemandKind::WorkBudgetExceeded, subject))
        }
        Stop::Read(Read::Execution(stop)) => query_execution_denied(subject, stop),
        Stop::ManagedIndex(cause) => {
            let caller_local = managed_authority_rejection(cause.cause());
            let denial = crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial::managed_source_query(cause, subject);
            if caller_local || matches!(denial.kind(), DemandKind::Cancelled | DemandKind::TimedOut)
            {
                request_admission_rejected(denial)
            } else {
                ProducerExecutionStop::ExecutionStopped(denial)
            }
        }
        other => ProducerExecutionStop::ExecutionStopped(failed(subject, other)),
    }
}

pub fn managed_authority_rejection(
    cause: crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexDenialCause<'_>,
) -> bool {
    use crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexDenialCause as Cause;
    match cause {
        Cause::CurrentControl(_) => true,
        Cause::SourceVerification {
            kind,
            authorization,
            execution,
        } => {
            use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind as Attempt;
            authorization.is_some()
                || execution.is_some()
                || matches!(
                    kind,
                    Attempt::Cancelled
                        | Attempt::DeadlineExceeded
                        | Attempt::DecisionFactBudgetExceeded
                        | Attempt::ExecutionRequest(_)
                )
        }
        Cause::Principal(_) | Cause::Scope(_) => true,
        Cause::QueryAdmission(stop) => admission_authority_rejection(stop.kind()),
        Cause::QueryRead(stop) => execution_authority_rejection(stop.kind()),
        _ => false,
    }
}

pub fn admission_authority_rejection(kind: AdmissionKind) -> bool {
    matches!(
        kind,
        AdmissionKind::Authorization(_)
            | AdmissionKind::StalePrincipal
            | AdmissionKind::ForeignPrincipal
            | AdmissionKind::ForeignScope
            | AdmissionKind::StaleScope
            | AdmissionKind::ScopeTypeMismatch
    )
}
pub fn execution_authority_rejection(kind: ExecutionKind) -> bool {
    matches!(
        kind,
        ExecutionKind::Authorization(_) | ExecutionKind::StalePrincipal | ExecutionKind::StaleScope
    )
}
