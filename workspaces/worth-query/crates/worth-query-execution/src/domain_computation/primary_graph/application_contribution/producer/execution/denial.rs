use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentityDenial;

mod source_readmission;
pub(super) use source_readmission::{query_admission_denied, query_execution_denied};

/// A rejected caller cannot terminally fail the output shared by other callers.
/// Only actual operation/query admission issues the first variant.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) enum ProducerExecutionStop
{
    RequestAdmissionDenied(ProducerRequestAdmissionRejection),
    ExecutionStopped(WorthQueryOutputDemandDenial),
    /// An exact selection whose producer declares no Preserve posture could
    /// not reuse its live output. Nothing ran and the shared row is not
    /// failed: the stop is this caller's.
    LiveOutputNotReused {
        producer: &'static str,
        reason: &'static str,
    },
}

impl ProducerExecutionStop {
    /// The typed refusal of a caller whose exact producer is not applicable
    /// to the live output it was selected for.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn live_output_not_reused(
        producer: &'static str,
        reason: &'static str,
    ) -> WorthQueryOutputDemandDenial {
        WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::MissingApplicableProducer,
            format!("{producer}: the exact producer declares no Preserve posture and cannot reuse its live output: {reason}"),
        )
    }
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct ProducerRequestAdmissionRejection
{
    denial: WorthQueryOutputDemandDenial,
}

impl ProducerRequestAdmissionRejection {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn into_denial(
        self,
    ) -> WorthQueryOutputDemandDenial {
        self.denial
    }
}

pub(super) fn request_admission_denied(
    subject: &str,
    error: impl std::fmt::Debug,
) -> ProducerExecutionStop {
    request_admission_rejected(failed(subject, error))
}

/// The request's principal did not resolve; the stop stays with the request.
pub(super) fn principal_rejected(
    subject: &str,
    error: crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial,
) -> ProducerExecutionStop {
    let kind = WorthQueryOutputDemandDenialKind::of_principal_resolution(error.kind());
    request_admission_rejected(denial(kind, format!("{subject}: {error:?}")))
}

/// The request's scope did not resolve; the stop stays with the request.
pub(super) fn scope_rejected(
    subject: &str,
    error: crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial,
) -> ProducerExecutionStop {
    let kind = WorthQueryOutputDemandDenialKind::of_scope_resolution(error.kind());
    request_admission_rejected(denial(kind, format!("{subject}: {error:?}")))
}

pub(super) fn request_authority_denied(
    subject: &str,
    error: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
) -> ProducerExecutionStop {
    request_admission_rejected(request_authority_stop(subject, error.kind()))
}

/// The stop of a request refused authorization.
pub(super) fn request_authority_stop(
    subject: &str,
    kind: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
) -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::of_request_authorization(kind),
        subject.to_owned(),
    )
}

pub(super) fn request_admission_rejected(
    denial: WorthQueryOutputDemandDenial,
) -> ProducerExecutionStop {
    ProducerExecutionStop::RequestAdmissionDenied(ProducerRequestAdmissionRejection { denial })
}

impl From<WorthQueryOutputDemandDenial> for ProducerExecutionStop {
    fn from(denial: WorthQueryOutputDemandDenial) -> Self {
        Self::ExecutionStopped(denial)
    }
}

/// A request that lost its authority mid-attempt stops only that request:
/// the row it claimed stays for a later advance.
pub(super) fn execution_failed(
    subject: &str,
    error: MutationHandlerExecutionDenial,
) -> ProducerExecutionStop {
    if let MutationHandlerExecutionDenial::Attempt(attempt) = &error {
        if let Some(authority) = attempt.request_authority() {
            return request_authority_denied(subject, authority.clone());
        }
    }
    if matches!(
        error,
        MutationHandlerExecutionDenial::Attempt(ref denial)
            if matches!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::SourceChanged
                    | WorthQueryApplicationAttemptDenialKind::SourceRetired
            )
    ) {
        denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            subject.to_owned(),
        )
        .into()
    } else {
        failed(subject, error).into()
    }
}

/// The producer's key or input did not encode; the denial names which one.
pub(super) fn identity_unavailable(
    subject: &str,
    error: ApplicationMutationIdentityDenial,
) -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        format!("{subject}: mutation identity unavailable: {error:?}"),
    )
}

pub(super) fn failed(subject: &str, error: impl std::fmt::Debug) -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        format!("{subject}: {error:?}"),
    )
}

pub(super) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<std::borrow::Cow<'static, str>>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
