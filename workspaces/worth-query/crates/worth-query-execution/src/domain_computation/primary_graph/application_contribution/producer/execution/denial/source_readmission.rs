//! Request rejection remains local to the caller through both query phases.

use super::WorthQueryOutputDemandDenialKind as DemandKind;
use super::{
    denial, failed, request_admission_denied, request_admission_rejected, request_authority_stop,
    ProducerExecutionStop,
};
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationOneShotDenialKind as ExecutionKind,
    WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind as AdmissionKind,
};

pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn query_admission_denied(
    subject: &str,
    error: WorthQueryApplicationQueryAdmissionDenial,
) -> ProducerExecutionStop {
    match error.kind() {
        AdmissionKind::StalePrincipal
        | AdmissionKind::ForeignPrincipal
        | AdmissionKind::ForeignScope
        | AdmissionKind::StaleScope
        | AdmissionKind::ScopeTypeMismatch => request_admission_denied(subject, error),
        AdmissionKind::Authorization(kind) => {
            request_admission_rejected(request_authority_stop(subject, kind))
        }
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
    match error.kind() {
        ExecutionKind::StalePrincipal | ExecutionKind::StaleScope => {
            request_admission_denied(subject, error)
        }
        ExecutionKind::Authorization(kind) => {
            request_admission_rejected(request_authority_stop(subject, kind))
        }
        ExecutionKind::Cancelled => {
            request_admission_rejected(denial(DemandKind::Cancelled, subject.to_owned()))
        }
        ExecutionKind::DeadlineExceeded => {
            request_admission_rejected(denial(DemandKind::TimedOut, subject.to_owned()))
        }
        _ => ProducerExecutionStop::ExecutionStopped(failed(subject, error)),
    }
}
