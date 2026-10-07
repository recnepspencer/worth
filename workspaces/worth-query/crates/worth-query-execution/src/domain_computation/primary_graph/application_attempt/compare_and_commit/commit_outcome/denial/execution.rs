//! Typed execution refusals before application publication.
use super::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitDenialKind as Kind,
    WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;

impl Denial {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_resource(
        denial: Resource,
        partition_identity: Option<u64>,
        policy_ancestor: Option<u32>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(
            Kind::ExecutionResource {
                denial,
                partition_identity,
                policy_ancestor,
            },
            detail,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_nested_stopped(
        partition_identity: Option<u64>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(
            Kind::ExecutionNestedPatternStopped { partition_identity },
            detail,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_worker_panicked(
        partition_identity: Option<u64>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(Kind::ExecutionWorkerPanicked { partition_identity }, detail)
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_unchecked_custom_kernel(
        partition_identity: Option<u64>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(
            Kind::ExecutionUncheckedCustomKernel { partition_identity },
            detail,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_identities_not_canonical(
        partition_identity: Option<u64>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(
            Kind::ExecutionIdentitiesNotCanonical { partition_identity },
            detail,
        )
    }
    fn execution_denial(kind: Kind, detail: impl Into<std::sync::Arc<str>>) -> Self {
        Self {
            kind,
            stage: Stage::ProviderCommit,
            detail: Some(detail.into()),
            cause: None,
        }
    }
}

impl Denial {
    /// Typed execution evidence when HEAD reported `ProviderRejected`.
    pub fn execution_denial_cause(
        &self,
    ) -> Option<
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    > {
        match &self.cause {
            Some(super::denial_cause::DenialCause::Execution(cause)) => Some(**cause),
            Some(super::denial_cause::DenialCause::CustomInvariant(_))
            | Some(super::denial_cause::DenialCause::RequestAuthority(_))
            | None => None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn provider_execution_denied(
        stage: Stage,
        cause: Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self {
            kind: Kind::ProviderRejected,
            stage,
            detail: Some(detail.into()),
            cause: Some(super::denial_cause::DenialCause::Execution(Box::new(cause))),
        }
    }
}
