use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::WorthQueryRuntimeError;

use crate::WorthServerOperationDenial;

use super::facade::{WorthServerDirectProjectionOutcome, WorthServerWorthNativeDirectFacade};

impl WorthServerWorthNativeDirectFacade {
    pub(super) fn runtime_error_outcome<T>(
        &self,
        error: WorthQueryRuntimeError,
    ) -> TransitionOutcome<
        T,
        crate::WorthServerQueryHandoffDenial,
        crate::WorthServerQueryHandoffDeferred,
        crate::WorthServerQueryHandoffStale,
        crate::WorthServerQueryHandoffRebindRequired,
        crate::WorthServerQueryHandoffFailure,
    > {
        match error {
            WorthQueryRuntimeError::MissingLiveView(_)
            | WorthQueryRuntimeError::MissingLiveSubscription(_) => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::RetainedQueryArtifactUnavailable,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
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
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationBindingDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::MutationContractDenied(_) => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationContractDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::ExistingTruthAssertionDenied(_) => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationAssertionDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::MutationContinuityDenied(_) => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationContinuityDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::MutationNamingDenied(_) => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationNamingDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            WorthQueryRuntimeError::MutationTargetReferenceDenied(_) => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationTargetReferenceDenied,
                    self.admission.request_context().diagnostics_profile(),
                    error.to_string(),
                ))
            }
            _ => TransitionOutcome::failed(crate::WorthServerQueryHandoffFailure::new(
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
    ) -> TransitionOutcome<
        T,
        crate::WorthServerQueryHandoffDenial,
        crate::WorthServerQueryHandoffDeferred,
        crate::WorthServerQueryHandoffStale,
        crate::WorthServerQueryHandoffRebindRequired,
        crate::WorthServerQueryHandoffFailure,
    > {
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
        TransitionOutcome::Denied(match facts {
            Some(facts) => denial.with_facts(facts),
            None => denial,
        })
    }

    pub(super) fn direct_mutation_scheduler_runtime_outcome<T>(
        &self,
        runtime_failure: &crate::WorthServerSchedulerRuntimeFailure,
    ) -> TransitionOutcome<
        T,
        crate::WorthServerQueryHandoffDenial,
        crate::WorthServerQueryHandoffDeferred,
        crate::WorthServerQueryHandoffStale,
        crate::WorthServerQueryHandoffRebindRequired,
        crate::WorthServerQueryHandoffFailure,
    > {
        match runtime_failure {
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationAssertionDenied { detail } => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationAssertionDenied,
                    self.admission.request_context().diagnostics_profile(),
                    detail.clone(),
                ))
            }
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationBindingDenied { detail } => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationBindingDenied,
                    self.admission.request_context().diagnostics_profile(),
                    detail.clone(),
                ))
            }
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationContractDenied { detail } => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationContractDenied,
                    self.admission.request_context().diagnostics_profile(),
                    detail,
                ))
            }
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationContinuityDenied {
                detail,
            } => TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                crate::WorthServerQueryHandoffDenialCode::DirectMutationContinuityDenied,
                self.admission.request_context().diagnostics_profile(),
                detail.clone(),
            )),
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationNamingDenied { detail } => {
                TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::DirectMutationNamingDenied,
                    self.admission.request_context().diagnostics_profile(),
                    detail.clone(),
                ))
            }
            crate::WorthServerSchedulerRuntimeFailure::DirectMutationTargetReferenceDenied {
                detail,
            } => TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                crate::WorthServerQueryHandoffDenialCode::DirectMutationTargetReferenceDenied,
                self.admission.request_context().diagnostics_profile(),
                detail.clone(),
            )),
            crate::WorthServerSchedulerRuntimeFailure::Opaque { .. }
            | crate::WorthServerSchedulerRuntimeFailure::BasisAdvancementDenied { .. }
            | crate::WorthServerSchedulerRuntimeFailure::BasisSnapshotReadDenied { .. } => {
                TransitionOutcome::failed(crate::WorthServerQueryHandoffFailure::new(
                    "direct_mutation_scheduler_runtime_failed",
                ))
            }
        }
    }
}
