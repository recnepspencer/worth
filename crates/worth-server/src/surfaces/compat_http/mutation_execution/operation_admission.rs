use super::WorthServerCompatibilityMutationOutcome;
use crate::{
    WorthServerCompatibilityFacade, WorthServerCompatibilityPreparedRequest,
    WorthServerOperationFamily, WorthServerOperationRequest, WorthServerPipelineInput,
    WorthServerPipelineIntent, WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode,
    WorthServerQueryHandoffFailure,
};
use worth_proof::TransitionOutcome;

pub(super) fn admit(
    facade: &WorthServerCompatibilityFacade,
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    operation_request: &WorthServerOperationRequest,
) -> WorthServerCompatibilityMutationOutcome<crate::WorthServerOperationAdmissionPosture> {
    let operation_name = operation_request.identity().operation_name().to_string();
    let diagnostics_profile = prepared_request
        .admission()
        .request_context()
        .diagnostics_profile();
    if let Err(denial) = facade.admit_operation_family_for_query(
        diagnostics_profile,
        WorthServerOperationFamily::QueryDirectSubmission,
    ) {
        return TransitionOutcome::Denied(denial);
    }
    let admission = match facade.middleware.admit(WorthServerPipelineInput::new(
        prepared_request
            .admission()
            .resolved_request_context()
            .clone(),
        WorthServerPipelineIntent::query_mutation(&operation_name),
    )) {
        TransitionOutcome::Success(value) => value,
        TransitionOutcome::Denied(value) => {
            return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::PreparedIntentMismatch,
                diagnostics_profile,
                value.detail(),
            ));
        }
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => {
            return TransitionOutcome::Failed(WorthServerQueryHandoffFailure::new(
                "compatibility_mutation_middleware_readmission_failed",
            ));
        }
    };
    let operation_admission =
        match crate::WorthServerOperationAdmissionFacade::with_operation_registry(
            facade.operation_registry.clone(),
        )
        .admit_declared(&admission, operation_request)
        {
            Ok(value) => value,
            Err(denial) => {
                return TransitionOutcome::Denied(
                    crate::surfaces::compat_http::map_operation_admission_denial(denial),
                );
            }
        };
    TransitionOutcome::Success(operation_admission)
}
