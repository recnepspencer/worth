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

pub(super) fn request_authority_denied(
    subject: &str,
    error: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
) -> ProducerExecutionStop {
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Kind;
    let refusal = match error.kind() {
        Kind::Cancelled => denial(
            WorthQueryOutputDemandDenialKind::Cancelled,
            subject.to_owned(),
        ),
        Kind::DeadlineExceeded => denial(
            WorthQueryOutputDemandDenialKind::TimedOut,
            subject.to_owned(),
        ),
        kind => denial(
            WorthQueryOutputDemandDenialKind::RequestAuthorization(kind),
            subject.to_owned(),
        ),
    };
    request_admission_rejected(refusal)
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

pub(super) fn execution_failed(
    subject: &str,
    error: MutationHandlerExecutionDenial,
) -> WorthQueryOutputDemandDenial {
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
    } else {
        failed(subject, error)
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
