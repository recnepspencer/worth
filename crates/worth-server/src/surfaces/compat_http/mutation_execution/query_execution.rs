use crate::{
    WorthServerCompatibilityFacade, WorthServerCompatibilityMutationPreconditionContext,
    WorthServerDirectContextArtifact, WorthServerDirectRemaskPosture, WorthServerOperationFamily,
    WorthServerOperationInputEnvelope, WorthServerOperationPlanner,
    WorthServerOperationPlannerInput, WorthServerOperationPreconditionPosture,
    WorthServerOperationReadinessFacade, WorthServerOperationRequestFacade,
    WorthServerPipelineInput, WorthServerPipelineIntent, WorthServerQueryHandoffDenial,
    WorthServerQueryHandoffDenialCode, WorthServerQueryHandoffDenialFacts,
    WorthServerQueryHandoffFailure, WorthServerQueryHandoffOperation,
    WorthServerQueryWorkspaceBindingRequest, WorthServerScheduledMutationResult,
};
use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily;

use super::{
    envelope::WorthServerCompatibilityMutationEnvelope,
    execution::{
        WorthServerCompatibilityMutationExecutionInput, WorthServerCompatibilityMutationOutcome,
    },
    idempotency::{WorthServerIdempotencyKey, WorthServerIdempotentRetryReceipt},
    query_execution_support::{
        canonical_mutation_request_digest, map_operation_request_denial, map_readiness_denial,
    },
    request::WorthServerCompatibilityMutationRequest,
    response::{WorthServerCompatibilityMutation, WorthServerCompatibilityMutationResult},
    retry_cache::{record_retry_completion, try_resolve_retry},
    schema::lower_query_operation,
};
impl WorthServerCompatibilityFacade {
    pub fn mutate(
        &self,
        input: WorthServerCompatibilityMutationExecutionInput,
    ) -> WorthServerCompatibilityMutationOutcome<WorthServerCompatibilityMutation> {
        let (prepared_request, operation_name, body) = input.into_parts();
        let operation_request =
            match WorthServerOperationRequestFacade::new(self.operation_registry.clone())
                .admit_from_compat_http(
                    &prepared_request,
                    WorthServerOperationFamily::QueryDirectSubmission,
                    &operation_name,
                    Some(WorthServerOperationInputEnvelope::json(
                        "compat-http.query-mutation.v1",
                        &body,
                    )),
                ) {
                Ok(value) => value,
                Err(denial) => {
                    return TransitionOutcome::Denied(map_operation_request_denial(denial));
                }
            };
        let diagnostics_profile = prepared_request
            .admission()
            .request_context()
            .diagnostics_profile();
        let mutation_request =
            match WorthServerCompatibilityMutationRequest::parse(body, diagnostics_profile) {
                Ok(value) => value,
                Err(denial) => return TransitionOutcome::Denied(denial),
            };
        execute_compatibility_mutation_request(
            self,
            prepared_request,
            operation_request,
            mutation_request,
        )
    }
}
pub(crate) fn execute_compatibility_mutation_request(
    facade: &WorthServerCompatibilityFacade,
    prepared_request: crate::WorthServerCompatibilityPreparedRequest,
    operation_request: crate::WorthServerOperationRequest,
    mutation_request: WorthServerCompatibilityMutationRequest,
) -> WorthServerCompatibilityMutationOutcome<WorthServerCompatibilityMutation> {
    let diagnostics_profile = operation_request
        .resolved_request_context()
        .request_context()
        .diagnostics_profile();
    let operation_name = operation_request.identity().operation_name().to_string();
    if let Err(denial) = facade.admit_operation_family_for_query(
        diagnostics_profile,
        WorthServerOperationFamily::QueryDirectSubmission,
    ) {
        return TransitionOutcome::Denied(denial);
    }
    let operation =
        match lower_query_operation(&operation_name, &mutation_request, diagnostics_profile) {
            Ok(value) => value,
            Err(denial) => return TransitionOutcome::Denied(denial),
        };
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
        .admit_declared(&admission, &operation_request)
        {
            Ok(value) => value,
            Err(denial) => {
                return TransitionOutcome::Denied(
                    crate::surfaces::compat_http::map_operation_admission_denial(denial),
                );
            }
        };
    let binding_request = WorthServerQueryWorkspaceBindingRequest::for_query_handoff(
        operation_admission
            .authorization_proof()
            .admission()
            .resolved_request_context()
            .clone(),
        WorthServerQueryHandoffOperation::query_mutation(&operation_name),
    );
    let bound_workspace = match facade
        .query_handoff
        .config()
        .workspace_provider()
        .bind_workspace(&binding_request)
    {
        Ok(workspace) => workspace,
        Err(error) => {
            return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::WorkspaceBindingFailed,
                diagnostics_profile,
                format!("{}: {}", error.stage(), error.message()),
            ));
        }
    };
    let observed_basis_digest = match bound_workspace.snapshot_identity() {
        Ok(snapshot_identity) => snapshot_identity.terminal_projection_for_reporting(),
        Err(denial) => {
            return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::WorkspaceBindingFailed,
                diagnostics_profile,
                format!("workspace snapshot identity: {denial}"),
            ));
        }
    };
    let readiness = WorthServerOperationReadinessFacade::with_operation_registry(
        facade.operation_registry.clone(),
    );
    let precondition = match readiness.evaluate_compatibility_mutation_preconditions(
        WorthServerCompatibilityMutationPreconditionContext::new(
            &prepared_request,
            &operation_name,
            mutation_request.canonical_digest(),
            &observed_basis_digest,
        ),
    ) {
        Ok(value) => value,
        Err(denial) => {
            return TransitionOutcome::Denied(map_readiness_denial(&prepared_request, denial));
        }
    };
    let plan = match WorthServerOperationPlanner::with_operation_registry(
        facade.query_handoff.config().clone(),
        facade.operation_registry.clone(),
    )
    .lower(
        WorthServerOperationPlannerInput::new(
            operation_admission,
            WorthServerQueryHandoffOperation::query_mutation_execution(operation.clone()),
        )
        .with_precondition_posture(
            WorthServerOperationPreconditionPosture::CompatibilityMutation(precondition.clone()),
        )
        .with_bound_workspace(bound_workspace),
    ) {
        Ok(value) => value,
        Err(denial) => return TransitionOutcome::Denied(denial.into_query_handoff_denial()),
    };
    let plan_proof = plan.proof();
    let query_handoff = plan.query_handoff();
    if let Err(error) = query_handoff
        .workspace()
        .admit_public_api_family(WorthQueryRuntimeFacadeFamily::Inspect)
    {
        return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily,
            diagnostics_profile,
            format!("query workspace does not admit `inspect` facade family: {error}"),
        ));
    }
    let support_posture = query_handoff.support_posture().clone();
    let workspace_name = query_handoff.workspace().name().to_string();
    let handoff_digest = query_handoff.canonical_digest().to_string();
    let idempotency_key = match WorthServerIdempotencyKey::from_prepared_request(&prepared_request)
    {
        Ok(value) => value,
        Err(denial) => return TransitionOutcome::Denied(denial),
    };
    let request_digest = canonical_mutation_request_digest(
        &prepared_request,
        &operation_name,
        &mutation_request,
        &precondition,
    );
    match try_resolve_retry(
        &facade.idempotency_store,
        &prepared_request,
        &idempotency_key,
        &request_digest,
    ) {
        Ok(Some(previously_completed)) => {
            return TransitionOutcome::Success(previously_completed);
        }
        Ok(None) => {}
        Err(denial) => return TransitionOutcome::Denied(denial),
    }

    let executed = match crate::WorthServerOperationScheduler::new(facade.responses.clone())
        .schedule_batch([plan])
    {
        Ok(batch) => batch.execute(),
        Err(denial) => {
            return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed,
                diagnostics_profile,
                denial.detail(),
            ))
        }
    };
    let outcome = &executed.outcomes()[0];
    if let Some(cancellation_posture) = outcome.cancellation_posture() {
        return TransitionOutcome::Failed(WorthServerQueryHandoffFailure::new(
            match cancellation_posture {
                crate::WorthServerSchedulerCancellationPosture::BeforeAdmission => {
                    "compatibility_mutation_cancelled_before_admission"
                }
                crate::WorthServerSchedulerCancellationPosture::AfterAdmissionBeforeExecution => {
                    "compatibility_mutation_cancelled_after_admission_before_execution"
                }
                crate::WorthServerSchedulerCancellationPosture::DuringExecution => {
                    "compatibility_mutation_cancelled_during_execution"
                }
            },
        ));
    }
    if let Some(failure_posture) = outcome.failure_posture() {
        return match failure_posture {
            crate::WorthServerSchedulerFailurePosture::StaleMutationBasis {
                expected_basis_digest,
                observed_basis_digest,
            } => TransitionOutcome::Denied(
                WorthServerQueryHandoffDenial::new(
                    WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed,
                    diagnostics_profile,
                    format!(
                        "compatibility mutation basis precondition `{expected_basis_digest}` did not match the scheduler-observed basis `{observed_basis_digest}`"
                    ),
                )
                .with_facts(
                    WorthServerQueryHandoffDenialFacts::default()
                        .with_basis_mismatch(expected_basis_digest, observed_basis_digest),
                ),
            ),
            crate::WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure {
                runtime_failure:
                    crate::WorthServerSchedulerRuntimeFailure::DirectMutationContractDenied {
                        detail,
                    },
            } => TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                diagnostics_profile,
                detail,
            )),
            crate::WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure { .. }
            | crate::WorthServerSchedulerFailurePosture::DependentSharedBasisFailure { .. }
            | crate::WorthServerSchedulerFailurePosture::OrderedLaneClosed { .. } => {
                TransitionOutcome::Failed(WorthServerQueryHandoffFailure::new(
                    "compatibility_mutation_scheduler_execution_failed",
                ))
            }
        };
    }
    let mutation_result = map_scheduled_mutation_result(
        outcome
            .mutation_result()
            .expect("scheduled compatibility mutation should carry a mutation result")
            .clone(),
    );
    let response_envelope = outcome
        .response_envelope()
        .expect("scheduled compatibility mutation should shape a response envelope")
        .clone();
    let direct_context = WorthServerDirectContextArtifact::new(
        prepared_request.admission().request_context(),
        &support_posture,
        &response_envelope,
        Some(&observed_basis_digest),
        WorthServerDirectRemaskPosture::visible(),
    );
    let retry_receipt = idempotency_key
        .as_ref()
        .map(|key| WorthServerIdempotentRetryReceipt::authoritative(key, &request_digest))
        .unwrap_or_else(|| WorthServerIdempotentRetryReceipt::Authoritative {
            idempotency_key: "none".to_string(),
            request_digest: request_digest.clone(),
            canonical_digest: format!(
                "compat-http-idempotent-retry-v1|class=executed|key:none|request:{}",
                request_digest
            ),
        });
    let envelope = WorthServerCompatibilityMutationEnvelope::new(
        support_posture,
        workspace_name,
        handoff_digest,
        direct_context,
        response_envelope,
        retry_receipt,
    );
    let mutation = WorthServerCompatibilityMutation::new(
        operation_request,
        plan_proof,
        mutation_request,
        precondition,
        mutation_result,
        envelope,
    );
    record_retry_completion(
        &facade.idempotency_store,
        &prepared_request,
        idempotency_key,
        request_digest,
        mutation.clone(),
    );
    TransitionOutcome::Success(mutation)
}
fn map_scheduled_mutation_result(
    mutation_result: WorthServerScheduledMutationResult,
) -> WorthServerCompatibilityMutationResult {
    match mutation_result {
        WorthServerScheduledMutationResult::Single {
            receipt,
            inspection,
        } => WorthServerCompatibilityMutationResult::Single {
            receipt,
            inspection,
        },
        WorthServerScheduledMutationResult::Batch {
            receipt,
            inspection,
        } => WorthServerCompatibilityMutationResult::Batch {
            receipt,
            inspection,
        },
    }
}
