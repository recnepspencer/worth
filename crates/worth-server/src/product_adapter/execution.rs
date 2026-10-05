use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod durable_recovery;
mod read_batch;

use crate::{
    product_operation_contract::{
        admit_retry, build_storage_key, record_retry, WorthServerProductIdempotencyBinding,
        WorthServerStoredProductOperation,
    },
    WorthServerAdmission, WorthServerCompatibilityPreparedRequest,
    WorthServerOperationAdmissionFacade, WorthServerOperationRegistry,
    WorthServerOperationRequestFacade, WorthServerProductSessionRegistry,
    WorthServerQueryHandoffConfig,
};

use super::execution_pipeline::{
    build_early_envelope, build_envelope, build_request_input, close_product_operation_readiness,
    declaration_metadata, validate_payload_schema, validate_product_mutation_preconditions,
    validate_success_result,
};
use super::{
    WorthServerCompletedProductOperation, WorthServerLoweredProductOperationPlan,
    WorthServerProductAdapterRegistry, WorthServerProductOperationAuthorization,
    WorthServerProductOperationAuthorizationRequest, WorthServerProductOperationDeclaration,
    WorthServerProductOperationExecutionBoundary, WorthServerProductOperationInput,
    WorthServerProductOperationOutcome, WorthServerProductOperationSurfaceDenial,
    WorthServerProductOperationSurfaceDenialCode, WorthServerProductOperationSurfaceDenialFacts,
    WorthServerScheduledProductOperation,
};

#[derive(Clone, Debug)]
pub struct WorthServerProductOperationRuntime {
    operation_registry: WorthServerOperationRegistry,
    adapter_registry: WorthServerProductAdapterRegistry,
    query_handoff_config: WorthServerQueryHandoffConfig,
    product_session_registry: WorthServerProductSessionRegistry,
    retry_store: Arc<Mutex<HashMap<String, WorthServerStoredProductOperation>>>,
    counters: Arc<crate::diagnostics::WorthServerCounters>,
}

impl WorthServerProductOperationRuntime {
    pub(crate) fn new(
        operation_registry: WorthServerOperationRegistry,
        adapter_registry: WorthServerProductAdapterRegistry,
        query_handoff_config: WorthServerQueryHandoffConfig,
        product_session_registry: WorthServerProductSessionRegistry,
        retry_store: Arc<Mutex<HashMap<String, WorthServerStoredProductOperation>>>,
        counters: Arc<crate::diagnostics::WorthServerCounters>,
    ) -> Self {
        Self {
            operation_registry,
            adapter_registry,
            query_handoff_config,
            product_session_registry,
            retry_store,
            counters,
        }
    }

    pub fn execute_from_worth_native(
        &self,
        admission: &WorthServerAdmission,
        input: WorthServerProductOperationInput,
    ) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial>
    {
        let (adapter, declaration) = self.resolve_declaration(input.operation_name())?;
        let request = WorthServerOperationRequestFacade::new(self.operation_registry.clone())
            .admit_from_worth_native_admission(admission, build_request_input(declaration, &input))
            .map_err(WorthServerProductOperationSurfaceDenial::from_request_denial)?;
        self.execute_resolved(
            admission,
            adapter.as_ref(),
            declaration,
            request,
            input.into_payload(),
        )
    }

    pub fn execute_from_compat_http(
        &self,
        prepared_request: &WorthServerCompatibilityPreparedRequest,
        input: WorthServerProductOperationInput,
    ) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial>
    {
        let (adapter, declaration) = self.resolve_declaration(input.operation_name())?;
        let request = WorthServerOperationRequestFacade::new(self.operation_registry.clone())
            .admit_from_compat_http_with_request_input(
                prepared_request,
                build_request_input(declaration, &input),
            )
            .map_err(WorthServerProductOperationSurfaceDenial::from_request_denial)?;
        self.execute_resolved(
            prepared_request.admission(),
            adapter.as_ref(),
            declaration,
            request,
            input.into_payload(),
        )
    }

    pub(crate) fn execute_from_product_protocol(
        &self,
        prepared_request: &WorthServerCompatibilityPreparedRequest,
        input: WorthServerProductOperationInput,
    ) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial>
    {
        let protocol_input = input.clone();
        match self.execute_from_compat_http(prepared_request, input) {
            Ok(operation) => Ok(operation),
            Err(denial) => self.project_session_denial(prepared_request, &protocol_input, denial),
        }
    }

    fn resolve_declaration(
        &self,
        operation_name: &str,
    ) -> Result<
        (
            &std::sync::Arc<dyn super::WorthServerProductApplicationAdapter>,
            &super::WorthServerProductOperationDeclaration,
        ),
        WorthServerProductOperationSurfaceDenial,
    > {
        self.adapter_registry
            .resolve(operation_name)
            .ok_or_else(|| {
                WorthServerProductOperationSurfaceDenial::new(
                    WorthServerProductOperationSurfaceDenialCode::UnknownOperationName,
                    format!("no registered product adapter owns `{operation_name}`"),
                )
            })
    }

    fn project_session_denial(
        &self,
        prepared_request: &WorthServerCompatibilityPreparedRequest,
        input: &WorthServerProductOperationInput,
        denial: WorthServerProductOperationSurfaceDenial,
    ) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial>
    {
        let Some(session_denial_code) = denial
            .facts()
            .and_then(WorthServerProductOperationSurfaceDenialFacts::session_denial_code)
        else {
            return Err(denial);
        };
        let (_, declaration) = self.resolve_declaration(input.operation_name())?;
        let request = WorthServerOperationRequestFacade::new(self.operation_registry.clone())
            .admit_from_compat_http_with_request_input(
                prepared_request,
                build_request_input(declaration, input),
            )
            .map_err(WorthServerProductOperationSurfaceDenial::from_request_denial)?;
        let outcome = WorthServerProductOperationOutcome::Denied(
            super::WorthServerProductOperationDenial::new(
                session_denial_code.reason_key(),
                denial.detail(),
            )
            .with_code(super::WorthServerProductOperationDenialCode::ProductSemantic),
        );
        let envelope = build_early_envelope(declaration.operation_name(), &request, &outcome);
        Ok(WorthServerCompletedProductOperation::new(outcome, envelope))
    }

    fn execute_resolved(
        &self,
        admission: &WorthServerAdmission,
        adapter: &dyn super::WorthServerProductApplicationAdapter,
        declaration: &super::WorthServerProductOperationDeclaration,
        request: crate::WorthServerOperationRequest,
        payload: super::WorthServerProductOperationPayload,
    ) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial>
    {
        if let Err(denial) = validate_payload_schema(declaration, &payload) {
            let outcome = WorthServerProductOperationOutcome::Denied(denial);
            let envelope = build_early_envelope(declaration.operation_name(), &request, &outcome);
            return Ok(WorthServerCompletedProductOperation::new(outcome, envelope));
        }
        let application_authorization = self.authorize_product_operation(declaration, &request)?;
        let admitted_session = request
            .identity()
            .product_session_identity()
            .and_then(|identity| {
                self.product_session_registry
                    .lookup(identity, request.resolved_request_context())
            });
        crate::product_session::admit_session_for_product_operation(
            admitted_session.as_ref(),
            &request,
            declaration.basis_kind(),
            matches!(
                declaration.authority_requirement(),
                super::WorthServerProductOperationAuthorityRequirement::DraftMutation { .. }
            ),
        )
        .map_err(|denial| {
            self.product_session_registry.record_denial(denial.code());
            WorthServerProductOperationSurfaceDenial::new(
                WorthServerProductOperationSurfaceDenialCode::AdmissionDenied,
                denial.detail().to_string(),
            )
            .with_facts(
                WorthServerProductOperationSurfaceDenialFacts::default()
                .with_session_denial_code(denial.code())
                .with_execution_boundary(
                    WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution,
                ),
            )
        })?;
        let durable_contract = declaration.durable_mutation_contract();
        validate_product_mutation_preconditions(
            &request,
            admitted_session.as_ref(),
            durable_contract.is_some(),
        )?;
        let retry_binding = if durable_contract.is_none() {
            request
                .identity()
                .idempotency_key()
                .map(|_| WorthServerProductIdempotencyBinding::derive(&request, &payload))
        } else {
            None
        };
        if let (Some(idempotency_key), Some(binding)) =
            (request.identity().idempotency_key(), retry_binding.as_ref())
        {
            let admitted_idempotency_key =
                crate::WorthServerProductIdempotencyKey::new(idempotency_key)
                    .expect("admitted operation request should preserve idempotency key validity");
            if let Some(previously_committed) = admit_retry(
                &self.retry_store,
                &build_storage_key(binding),
                &admitted_idempotency_key,
                binding.request_digest(),
            )? {
                return Ok(previously_committed);
            }
        }
        let admission = WorthServerOperationAdmissionFacade::with_operation_registry(
            self.operation_registry.clone(),
        )
        .admit(
            admission,
            &request,
            declaration_metadata(declaration, &request)?,
        )
        .map_err(WorthServerProductOperationSurfaceDenial::from_admission_denial)?;
        let readiness = close_product_operation_readiness(
            &self.operation_registry,
            &self.query_handoff_config,
            &admission,
            declaration,
            request.resolved_request_context(),
        )?;
        let plan = WorthServerLoweredProductOperationPlan::new(
            admission,
            declaration.clone(),
            payload,
            application_authorization,
            readiness.support_posture().clone(),
            readiness.precondition_posture().clone(),
            readiness.concurrency_class(),
        );
        let scheduled = WorthServerScheduledProductOperation::admit(plan, admitted_session)?;
        if let Some(durable_contract) = durable_contract {
            let executor = self
                .adapter_registry
                .resolve_durable_executor(declaration.operation_name())
                .ok_or_else(|| {
                    WorthServerProductOperationSurfaceDenial::new(
                        WorthServerProductOperationSurfaceDenialCode::InvalidDeclaration,
                        "validated durable product declaration lost its installed executor"
                            .to_string(),
                    )
                })?;
            return self.adapter_registry.coordinate_mutation_lane(
                scheduled.scheduler_admission().scheduler_lane(),
                || {
                    crate::durable_product_mutation::execute_durable_product_mutation(
                        executor.as_ref(),
                        &scheduled,
                        durable_contract,
                        &self.counters,
                    )
                },
            );
        }
        let outcome = match adapter.execute(&scheduled) {
            Ok(success) => {
                validate_success_result(declaration, &success)?;
                self.counters
                    .record_product_result_artifact(success.result_artifact().body().byte_len());
                WorthServerProductOperationOutcome::Success(success)
            }
            Err(super::WorthServerProductAdapterExecutionError::InvalidResultArtifact(error)) => {
                if error.code()
                    == crate::WorthServerProductResultArtifactErrorCode::InlineBudgetExceeded
                {
                    self.counters.increment_product_result_oversized_denials();
                }
                return Err(WorthServerProductOperationSurfaceDenial::new(
                    WorthServerProductOperationSurfaceDenialCode::InvalidResultArtifact,
                    error.detail().to_string(),
                )
                .with_facts(
                    WorthServerProductOperationSurfaceDenialFacts::default()
                        .with_execution_boundary(
                            WorthServerProductOperationExecutionBoundary::AdapterExecutionAttempted,
                        ),
                ));
            }
            Err(error) => declaration.error_map().map_error(error),
        };
        let envelope = build_envelope(&scheduled, &outcome);
        let mut completed = WorthServerCompletedProductOperation::new(outcome, envelope)
            .with_scheduled_operation(&scheduled);
        if let (Some(idempotency_key), Some(binding)) =
            (request.identity().idempotency_key(), retry_binding.as_ref())
        {
            completed = completed.with_retry_receipt(
                crate::WorthServerProductOperationRetryReceipt::executed(
                    idempotency_key,
                    binding.request_digest(),
                ),
            );
            record_retry(
                &self.retry_store,
                build_storage_key(binding),
                binding.request_digest().to_string(),
                completed.clone(),
            );
        }
        Ok(completed)
    }

    fn authorize_product_operation(
        &self,
        declaration: &WorthServerProductOperationDeclaration,
        request: &crate::WorthServerOperationRequest,
    ) -> Result<
        Option<WorthServerProductOperationAuthorization>,
        WorthServerProductOperationSurfaceDenial,
    > {
        let Some(authorizer) = self.adapter_registry.operation_authorizer() else {
            return Ok(None);
        };
        authorizer
            .authorize(&WorthServerProductOperationAuthorizationRequest::new(
                declaration.operation_name(),
                request,
            ))
            .map(Some)
            .map_err(|denial| {
                WorthServerProductOperationSurfaceDenial::new(
                    WorthServerProductOperationSurfaceDenialCode::AdmissionDenied,
                    format!("{}: {}", denial.reason_key(), denial.detail()),
                )
                .with_facts(
                    WorthServerProductOperationSurfaceDenialFacts::default()
                        .with_execution_boundary(
                            WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution,
                        ),
                )
            })
    }
}
