use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::WorthQueryRuntimeError;

use crate::WorthServerOperationDenial;

use super::facade::{WorthServerDirectProjectionOutcome, WorthServerWorthNativeDirectFacade};

type DirectQueryOutcome<T> = TransitionOutcome<
    T,
    crate::WorthServerQueryHandoffDenial,
    crate::WorthServerQueryHandoffDeferred,
    crate::WorthServerQueryHandoffStale,
    crate::WorthServerQueryHandoffRebindRequired,
    crate::WorthServerQueryHandoffFailure,
>;

/// Denial and failure carry no success value. One compiled match serves every
/// caller; the shell only places that fact in the caller's outcome type.
enum DirectQueryHandoff {
    Denied(crate::WorthServerQueryHandoffDenial),
    Failed(crate::WorthServerQueryHandoffFailure),
}

impl DirectQueryHandoff {
    fn into_outcome<T>(self) -> DirectQueryOutcome<T> {
        match self {
            Self::Denied(denial) => TransitionOutcome::Denied(denial),
            Self::Failed(failure) => TransitionOutcome::failed(failure),
        }
    }
}

impl WorthServerWorthNativeDirectFacade {
    pub(super) fn runtime_error_outcome<T>(
        &self,
        error: WorthQueryRuntimeError,
    ) -> DirectQueryOutcome<T> {
        self.runtime_error_handoff(error).into_outcome()
    }

    #[inline(never)]
    fn runtime_error_handoff(&self, error: WorthQueryRuntimeError) -> DirectQueryHandoff {
        match error {
            WorthQueryRuntimeError::MissingLiveView(_)
            | WorthQueryRuntimeError::MissingLiveSubscription(_) => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::RetainedQueryArtifactUnavailable,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily,
                    self.admission.request_context().diagnostics_profile(),
                    format!(
                        "query workspace does not admit `{}` facade family: {}",
                        denial.family().as_str(),
                        denial.reason()
                    ),
                ))
            }
            WorthQueryRuntimeError::MutationBindingDenied(_) => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationBindingDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::MutationContractDenied(_) => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationContractDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::ExistingTruthAssertionDenied(_) => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationAssertionDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::MutationContinuityDenied(_) => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationContinuityDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::MutationNamingDenied(_) => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationNamingDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::MutationTargetReferenceDenied(_) => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationTargetReferenceDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            _ => DirectQueryHandoff::Failed(crate::WorthServerQueryHandoffFailure::new(
                "direct_query_execution_failed",
            )),
        }
    }

    pub(super) fn projection_error_outcome(
        &self,
        error: worth_query::facade::foundation::ProjectionFactConsumptionPathError,
    ) -> WorthServerDirectProjectionOutcome {
        match error {
            worth_query::facade::foundation::ProjectionFactConsumptionPathError::Declaration(
                detail,
            ) => TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                crate::WorthServerQueryHandoffDenialCode::DirectProjectionBindingInvalid,
                self.admission.request_context().diagnostics_profile(),
                format!("{detail:?}"),
            )),
            worth_query::facade::foundation::ProjectionFactConsumptionPathError::Extraction(_) => {
                TransitionOutcome::failed(crate::WorthServerQueryHandoffFailure::new(
                    "direct_projection_extraction_failed",
                ))
            }
        }
    }

    pub(super) fn operation_denial_outcome<T>(
        &self,
        denial: WorthServerOperationDenial,
    ) -> DirectQueryOutcome<T> {
        self.operation_denial_handoff(denial).into_outcome()
    }

    #[inline(never)]
    fn operation_denial_handoff(&self, denial: WorthServerOperationDenial) -> DirectQueryHandoff {
        let (code, facts) = match denial {
            WorthServerOperationDenial::UnregisteredFamily { .. } => (
                crate::WorthServerQueryHandoffDenialCode::OperationFamilyNotRegistered,
                None,
            ),
            WorthServerOperationDenial::DisabledFamily { .. } => (
                crate::WorthServerQueryHandoffDenialCode::OperationFamilyDisabled,
                None,
            ),
            WorthServerOperationDenial::SurfaceFamilyNotExposed { .. } => (
                crate::WorthServerQueryHandoffDenialCode::OperationFamilyNotExposedOnSurface,
                None,
            ),
            WorthServerOperationDenial::UnknownOperationName {
                ref operation_name, ..
            } => (
                crate::WorthServerQueryHandoffDenialCode::UnknownOperationName,
                Some(
                    crate::WorthServerQueryHandoffDenialFacts::default()
                        .with_rejected_operation_name(operation_name.clone()),
                ),
            ),
        };
        let denial = crate::WorthServerQueryHandoffDenial::new(
            code,
            self.admission.request_context().diagnostics_profile(),
            denial.detail(),
        );
        DirectQueryHandoff::Denied(match facts {
            Some(facts) => denial.with_facts(facts),
            None => denial,
        })
    }

    pub(super) fn direct_mutation_scheduler_runtime_outcome<T>(
        &self,
        runtime_failure: &crate::WorthServerSchedulerRuntimeFailure,
    ) -> DirectQueryOutcome<T> {
        self.direct_mutation_scheduler_runtime_handoff(runtime_failure)
            .into_outcome()
    }

    #[inline(never)]
    fn direct_mutation_scheduler_runtime_handoff(
        &self,
        runtime_failure: &crate::WorthServerSchedulerRuntimeFailure,
    ) -> DirectQueryHandoff {
        match runtime_failure {
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationAssertionDenied { detail } => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationAssertionDenied,
                    self.admission.request_context().diagnostics_profile(),
                    detail.clone(),
                ))
            }
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationBindingDenied { detail } => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationBindingDenied,
                    self.admission.request_context().diagnostics_profile(),
                    detail.clone(),
                ))
            }
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationContractDenied { detail } => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationContractDenied,
                    self.admission.request_context().diagnostics_profile(),
                    detail,
                ))
            }
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationContinuityDenied {
                detail,
            } => DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                crate::WorthServerQueryHandoffDenialCode::DirectMutationContinuityDenied,
                self.admission.request_context().diagnostics_profile(),
                detail.clone(),
            )),
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationNamingDenied { detail } => {
                DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationNamingDenied,
                    self.admission.request_context().diagnostics_profile(),
                    detail.clone(),
                ))
            }
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationTargetReferenceDenied {
                detail,
            } => DirectQueryHandoff::Denied(crate::WorthServerQueryHandoffDenial::new(
                crate::WorthServerQueryHandoffDenialCode::DirectMutationTargetReferenceDenied,
                self.admission.request_context().diagnostics_profile(),
                detail.clone(),
            )),
            crate::WorthServerSchedulerRuntimeFailure::Opaque { .. }
            | crate::WorthServerSchedulerRuntimeFailure::BasisAdvancementDenied { .. }
            | crate::WorthServerSchedulerRuntimeFailure::BasisSnapshotReadDenied { .. } => {
                DirectQueryHandoff::Failed(crate::WorthServerQueryHandoffFailure::new(
                    "direct_mutation_scheduler_runtime_failed",
                ))
            }
        }
    }
}
