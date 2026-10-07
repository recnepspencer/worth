use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionResourceLease};

mod checked;

use crate::{
    WorthServerAdmission, WorthServerCompletedProductOperation,
    WorthServerExecutedProductReadBatch, WorthServerLoweredProductOperationPlan,
    WorthServerOperationAdmissionFacade, WorthServerOperationFamily, WorthServerOperationRegistry,
    WorthServerOperationRequestFacade, WorthServerOperationSchedulerCounters,
    WorthServerProductAdapterRegistry, WorthServerProductApplicationAdapter,
    WorthServerProductOperationAuthorizationRequest, WorthServerProductOperationDeclaration,
    WorthServerProductOperationExecutionBoundary, WorthServerProductOperationInput,
    WorthServerProductOperationOutcome, WorthServerProductOperationSurfaceDenial,
    WorthServerProductOperationSurfaceDenialCode, WorthServerProductOperationSurfaceDenialFacts,
    WorthServerProductReadBatchStop, WorthServerQueryHandoffConfig,
    WorthServerScheduledProductOperation,
};

use super::{
    build_envelope, build_request_input, close_product_operation_readiness, declaration_metadata,
    validate_payload_schema, validate_success_result,
};

pub(in crate::product_adapter) fn execute_shared_read_batch_from_worth_native(
    operation_registry: &WorthServerOperationRegistry,
    adapter_registry: &WorthServerProductAdapterRegistry,
    query_handoff_config: &WorthServerQueryHandoffConfig,
    admission: &WorthServerAdmission,
    inputs: Vec<WorthServerProductOperationInput>,
) -> Result<WorthServerExecutedProductReadBatch, WorthServerProductOperationSurfaceDenial> {
    let prepared = inputs
        .into_iter()
        .map(|input| {
            prepare_shared_read_slot(
                operation_registry,
                adapter_registry,
                query_handoff_config,
                admission,
                input,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut counters = WorthServerOperationSchedulerCounters::default();
    counters.set_planned_batch_width(prepared.len());
    counters.increment_admitted_read_slot_count_by(prepared.len());
    counters.increment_queued_read_slot_count_by(prepared.len());
    let operations = prepared
        .into_iter()
        .map(execute_shared_read_slot)
        .collect::<Result<Vec<_>, _>>()?;
    counters.increment_completed_read_slot_count_by(operations.len());
    Ok(WorthServerExecutedProductReadBatch::new(
        operations, counters,
    ))
}

pub(in crate::product_adapter) fn execute_shared_read_batch_with_lease_from_worth_native(
    operation_registry: &WorthServerOperationRegistry,
    adapter_registry: &WorthServerProductAdapterRegistry,
    query_handoff_config: &WorthServerQueryHandoffConfig,
    admission: &WorthServerAdmission,
    inputs: Vec<WorthServerProductOperationInput>,
    lease: &ExecutionResourceLease<'_>,
) -> Result<WorthServerExecutedProductReadBatch, WorthServerProductReadBatchStop> {
    checked::execute(
        operation_registry,
        adapter_registry,
        query_handoff_config,
        admission,
        inputs,
        lease,
    )
}

pub(super) struct WorthServerPreparedProductReadSlot {
    adapter: Arc<dyn WorthServerProductApplicationAdapter>,
    declaration: WorthServerProductOperationDeclaration,
    scheduled: WorthServerScheduledProductOperation,
}

impl WorthServerPreparedProductReadSlot {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.declaration
            .owned_allocation_capacity_bytes()
            .saturating_add(self.scheduled.owned_allocation_capacity_bytes())
    }
}

struct PreparedReadPacket {
    slot: Arc<WorthServerPreparedProductReadSlot>,
}

impl ChargedBytes for PreparedReadPacket {
    fn additional_charged_bytes(&self) -> u64 {
        self.slot.owned_allocation_capacity_bytes().saturating_add(
            u64::try_from(
                std::mem::size_of::<WorthServerPreparedProductReadSlot>()
                    + 2 * std::mem::size_of::<usize>(),
            )
            .unwrap_or(u64::MAX),
        )
    }
}

struct AdapterReadResult {
    outcome: Result<
        crate::WorthServerProductOperationSuccess,
        crate::WorthServerProductAdapterExecutionError,
    >,
}

impl ChargedBytes for AdapterReadResult {
    fn additional_charged_bytes(&self) -> u64 {
        match &self.outcome {
            Ok(success) => success.owned_allocation_capacity_bytes(),
            Err(error) => error.owned_allocation_capacity_bytes(),
        }
    }
}

fn finalize_shared_read_slot(
    ordinal: usize,
    slot: &WorthServerPreparedProductReadSlot,
    result: AdapterReadResult,
) -> Result<WorthServerCompletedProductOperation, WorthServerProductReadBatchStop> {
    let outcome = match result.outcome {
        Ok(success) => {
            validate_success_result(&slot.declaration, &success).map_err(|denial| {
                WorthServerProductReadBatchStop::Finalization { ordinal, denial }
            })?;
            WorthServerProductOperationOutcome::Success(success)
        }
        Err(error) => slot.declaration.error_map().map_error(error),
    };
    let envelope = build_envelope(&slot.scheduled, &outcome);
    Ok(WorthServerCompletedProductOperation::new(outcome, envelope)
        .with_scheduled_operation(&slot.scheduled))
}

fn prepare_shared_read_slot(
    operation_registry: &WorthServerOperationRegistry,
    adapter_registry: &WorthServerProductAdapterRegistry,
    query_handoff_config: &WorthServerQueryHandoffConfig,
    admission: &WorthServerAdmission,
    input: WorthServerProductOperationInput,
) -> Result<WorthServerPreparedProductReadSlot, WorthServerProductOperationSurfaceDenial> {
    let (adapter, declaration) = adapter_registry
        .resolve(input.operation_name())
        .ok_or_else(|| {
            WorthServerProductOperationSurfaceDenial::new(
                WorthServerProductOperationSurfaceDenialCode::UnknownOperationName,
                format!(
                    "no registered product adapter owns `{}`",
                    input.operation_name()
                ),
            )
        })?;
    if declaration.operation_family() != WorthServerOperationFamily::ProductApplicationRead {
        return Err(WorthServerProductOperationSurfaceDenial::new(
            WorthServerProductOperationSurfaceDenialCode::InvalidDeclaration,
            "shared-read batch only admits product read operations".to_string(),
        ));
    }
    let request_input = build_request_input(declaration, &input);
    let payload = input.into_payload();
    validate_payload_schema(declaration, &payload).map_err(|denial| {
        WorthServerProductOperationSurfaceDenial::new(
            WorthServerProductOperationSurfaceDenialCode::InvalidDeclaration,
            format!(
                "shared-read batch requires scheduler-backed read proofs, but payload schema validation denied early: {}",
                denial.reason_key()
            ),
        )
    })?;
    let request = WorthServerOperationRequestFacade::new(operation_registry.clone())
        .admit_from_worth_native_admission(admission, request_input)
        .map_err(WorthServerProductOperationSurfaceDenial::from_request_denial)?;
    let application_authorization = adapter_registry
        .operation_authorizer()
        .map(|authorizer| {
            authorizer.authorize(&WorthServerProductOperationAuthorizationRequest::new(
                declaration.operation_name(),
                &request,
            ))
        })
        .transpose()
        .map_err(|denial| {
            WorthServerProductOperationSurfaceDenial::new(
                WorthServerProductOperationSurfaceDenialCode::AdmissionDenied,
                format!("{}: {}", denial.reason_key(), denial.detail()),
            )
            .with_facts(
                WorthServerProductOperationSurfaceDenialFacts::default().with_execution_boundary(
                    WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution,
                ),
            )
        })?;
    let operation_admission =
        WorthServerOperationAdmissionFacade::with_operation_registry(operation_registry.clone())
            .admit(
                admission,
                &request,
                declaration_metadata(declaration, &request)?,
            )
            .map_err(WorthServerProductOperationSurfaceDenial::from_admission_denial)?;
    let readiness = close_product_operation_readiness(
        operation_registry,
        query_handoff_config,
        &operation_admission,
        declaration,
        request.resolved_request_context(),
    )?;
    let scheduled = WorthServerScheduledProductOperation::admit(
        WorthServerLoweredProductOperationPlan::new(
            operation_admission,
            declaration.clone(),
            payload,
            application_authorization,
            readiness.support_posture().clone(),
            readiness.precondition_posture().clone(),
            readiness.concurrency_class(),
        ),
        None,
    )?;
    if scheduled.scheduler_admission().scheduler_lane() != "shared-read" {
        return Err(WorthServerProductOperationSurfaceDenial::new(
            WorthServerProductOperationSurfaceDenialCode::InvalidDeclaration,
            "shared-read batch requires shared-read scheduler admission".to_string(),
        ));
    }
    Ok(WorthServerPreparedProductReadSlot {
        adapter: adapter.clone(),
        declaration: declaration.clone(),
        scheduled,
    })
}

fn execute_shared_read_slot(
    slot: WorthServerPreparedProductReadSlot,
) -> Result<WorthServerCompletedProductOperation, WorthServerProductOperationSurfaceDenial> {
    let outcome = match slot.adapter.execute(&slot.scheduled) {
        Ok(success) => {
            validate_success_result(&slot.declaration, &success)?;
            WorthServerProductOperationOutcome::Success(success)
        }
        Err(error) => slot.declaration.error_map().map_error(error),
    };
    let envelope = build_envelope(&slot.scheduled, &outcome);
    Ok(WorthServerCompletedProductOperation::new(outcome, envelope)
        .with_scheduled_operation(&slot.scheduled))
}
