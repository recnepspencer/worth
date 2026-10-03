//! Request rejection remains local to the caller through both query phases.

use super::WorthQueryOutputDemandDenialKind as DemandKind;
use super::{
    denial, failed, request_admission_denied, request_admission_rejected, ProducerExecutionStop,
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
        | AdmissionKind::ScopeTypeMismatch
        | AdmissionKind::Authorization(_) => request_admission_denied(subject, error),
        AdmissionKind::Cancelled => {
            request_admission_rejected(denial(DemandKind::Cancelled, subject))
        }
        AdmissionKind::DeadlineExceeded => {
            request_admission_rejected(denial(DemandKind::TimedOut, subject))
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
        ExecutionKind::StalePrincipal
        | ExecutionKind::StaleScope
        | ExecutionKind::Authorization(_) => request_admission_denied(subject, error),
        ExecutionKind::Cancelled => {
            request_admission_rejected(denial(DemandKind::Cancelled, subject))
        }
        ExecutionKind::DeadlineExceeded => {
            request_admission_rejected(denial(DemandKind::TimedOut, subject))
        }
        _ => ProducerExecutionStop::ExecutionStopped(failed(subject, error)),
    }
}
