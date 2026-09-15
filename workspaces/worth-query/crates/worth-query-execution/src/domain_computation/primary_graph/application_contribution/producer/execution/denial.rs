use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::{
    MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenialKind,
};

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
        denial(WorthQueryOutputDemandDenialKind::Superseded, subject)
    } else {
        failed(subject, error)
    }
}

pub(super) fn failed(subject: &str, error: impl std::fmt::Debug) -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        format!("{subject}: {error:?}"),
    )
}

pub(super) fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
