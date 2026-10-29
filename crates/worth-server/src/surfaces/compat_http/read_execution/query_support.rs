use worth_proof::TransitionOutcome;

use crate::{
    WorthServerAdmittedDirectDeclaration, WorthServerCompatibilityFacade,
    WorthServerCompatibilityPreparedRequest, WorthServerLoweredOperationPlan,
    WorthServerOperationPlanner, WorthServerOperationPlannerInput, WorthServerQueryHandoffDenial,
    WorthServerQueryHandoffFailure, WorthServerQueryHandoffOperation, WorthServerReadValidator,
};

use super::{
    basis::WorthServerExternalBasisRequest,
    conditional::WorthServerConditionalRead,
    execution::{
        admit_declaration, named_read_declaration, WorthServerCompatibilityExecutionOutcome,
    },
};

pub(super) fn compatibility_basis_request(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
) -> Result<WorthServerExternalBasisRequest, WorthServerQueryHandoffDenial> {
    WorthServerExternalBasisRequest::from_prepared_request(prepared_request)
}

pub(super) fn admitted_named_read_declaration(
    facade: &WorthServerCompatibilityFacade,
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    operation_name: &str,
) -> Result<WorthServerAdmittedDirectDeclaration, WorthServerQueryHandoffDenial> {
    admit_declaration(
        &facade.declaration_intake,
        prepared_request.admission().clone(),
        named_read_declaration(operation_name),
    )
}

pub(super) fn compatibility_plan(
    facade: &WorthServerCompatibilityFacade,
    operation_admission: crate::WorthServerOperationAdmissionPosture,
    operation: WorthServerQueryHandoffOperation,
) -> Result<WorthServerLoweredOperationPlan, WorthServerQueryHandoffDenial> {
    WorthServerOperationPlanner::with_operation_registry(
        facade.query_handoff.config().clone(),
        facade.operation_registry.clone(),
    )
    .lower(WorthServerOperationPlannerInput::new(
        operation_admission,
        operation,
    ))
    .map_err(crate::WorthServerOperationPlanDenial::into_query_handoff_denial)
}

pub(super) fn validate_conditional_read(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    conditional_read: &WorthServerConditionalRead,
    validator: &WorthServerReadValidator,
) -> Result<(), WorthServerQueryHandoffDenial> {
    if let Some(expected) = conditional_read.if_match()
        && expected != validator.entity_tag()
    {
        return Err(crate::WorthServerQueryHandoffDenial::new(
            crate::WorthServerQueryHandoffDenialCode::CompatibilityConditionalReadPreconditionFailed,
            prepared_request.admission().request_context().diagnostics_profile(),
            "compatibility if-match validator does not match the canonical read validator",
        ));
    }
    if let Some(expected) = conditional_read.if_none_match()
        && expected == validator.entity_tag()
    {
        return Err(crate::WorthServerQueryHandoffDenial::new(
            crate::WorthServerQueryHandoffDenialCode::CompatibilityConditionalReadNotModified,
            prepared_request
                .admission()
                .request_context()
                .diagnostics_profile(),
            "compatibility if-none-match validator already matches the canonical read validator",
        ));
    }
    Ok(())
}

pub(super) fn runtime_error_outcome<T>(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    error: worth_query::facade::runtime::WorthQueryRuntimeError,
) -> WorthServerCompatibilityExecutionOutcome<T> {
    compatibility_runtime_error(prepared_request, error).into_outcome()
}

enum CompatibilityRuntimeError {
    Denied(WorthServerQueryHandoffDenial),
    Failed(WorthServerQueryHandoffFailure),
}

impl CompatibilityRuntimeError {
    fn into_outcome<T>(self) -> WorthServerCompatibilityExecutionOutcome<T> {
        match self {
            Self::Denied(denial) => TransitionOutcome::Denied(denial),
            Self::Failed(failure) => TransitionOutcome::Failed(failure),
        }
    }
}

#[inline(never)]
fn compatibility_runtime_error(
    prepared_request: &WorthServerCompatibilityPreparedRequest,
    error: worth_query::facade::runtime::WorthQueryRuntimeError,
) -> CompatibilityRuntimeError {
    match error {
        worth_query::facade::runtime::WorthQueryRuntimeError::MissingLiveView(_)
        | worth_query::facade::runtime::WorthQueryRuntimeError::MissingLiveSubscription(_) => {
            CompatibilityRuntimeError::Denied(crate::WorthServerQueryHandoffDenial::new(
                crate::WorthServerQueryHandoffDenialCode::RetainedQueryArtifactUnavailable,
                prepared_request
                    .admission()
                    .request_context()
                    .diagnostics_profile(),
                error.to_string(),
            ))
        }
        _ => CompatibilityRuntimeError::Failed(WorthServerQueryHandoffFailure::new(
            "compatibility_query_execution_failed",
        )),
    }
}
