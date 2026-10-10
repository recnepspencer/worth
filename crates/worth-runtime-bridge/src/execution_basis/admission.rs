use crate::facade::RuntimeBridge;
use crate::snapshot::PlannedTruthViewPacket;
use crate::source::{
    with_async_request_signal_runtime, AdmittedBridgeAsyncRequestIdentity,
    BridgeAsyncRequestAdmissionRequest, BridgeAsyncRequestTruthViewBasis,
    BridgeAsyncRequestTruthViewBasisKind, ValidatedBridgeAsyncRequestBasisBinding,
};

use super::authority::BridgeBoundExecutionBasisParts;
use super::managed_declaration::{atomic_execution_declaration, managed_execution_declaration};
use super::posture::{BridgeExecutionBasisPosture, BridgeManagedExecutionBasis};
use super::reservation::BridgeExecutionBasisReservationKey;
use super::{
    BridgeBoundExecutionBasis, BridgeExecutionBasisCounters, BridgeExecutionBasisDenial,
    BridgeExecutionBasisDenialKind, BridgeManagedExecutionIntent,
    BridgeManagedExecutionStepContract,
};
use worth_signal::facade::{ResourceCancellationReason, ResourceManagedQueueBinding};

pub(crate) fn admit_managed_execution_basis(
    runtime: &RuntimeBridge,
    intent: BridgeManagedExecutionIntent,
    step_contract: BridgeManagedExecutionStepContract,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    planned: PlannedTruthViewPacket,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeBoundExecutionBasis, BridgeExecutionBasisDenial> {
    admit_execution_basis(
        runtime,
        intent,
        Some(step_contract),
        truth_basis,
        planned,
        execution,
    )
}

pub(crate) fn admit_atomic_execution_basis(
    runtime: &RuntimeBridge,
    intent: BridgeManagedExecutionIntent,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    planned: PlannedTruthViewPacket,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeBoundExecutionBasis, BridgeExecutionBasisDenial> {
    admit_execution_basis(runtime, intent, None, truth_basis, planned, execution)
}

fn admit_execution_basis(
    runtime: &RuntimeBridge,
    intent: BridgeManagedExecutionIntent,
    step_contract: Option<BridgeManagedExecutionStepContract>,
    truth_basis: BridgeAsyncRequestTruthViewBasis,
    planned: PlannedTruthViewPacket,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeBoundExecutionBasis, BridgeExecutionBasisDenial> {
    let mut counters = BridgeExecutionBasisCounters::default();
    counters.checked_managed_intent();
    validate_managed_intent(&intent, &counters)?;
    counters.checked_truth_basis();
    validate_truth_basis(&truth_basis, &planned, &counters)?;

    counters.checked_reservation();
    let reservation = reserve_intent(runtime, &intent, &counters)?;
    counters.materialized_truth();
    let observation = runtime
        .materialize_truth_view_observation(planned, execution)
        .map_err(|error| {
            denial(
                BridgeExecutionBasisDenialKind::TruthMaterializationFailed,
                error.to_string(),
                &counters,
            )
        })?;

    let lowered = match &step_contract {
        Some(contract) => {
            managed_execution_declaration(intent.identity().as_str(), contract.deadline_nanos())
        }
        None => atomic_execution_declaration(intent.identity().as_str()),
    }
    .map_err(|error| {
        denial(
            BridgeExecutionBasisDenialKind::SignalDeclarationUnavailable,
            error.detail().to_owned(),
            &counters,
        )
    })?;
    let basis_binding = ValidatedBridgeAsyncRequestBasisBinding::bind(&lowered, truth_basis);
    let request = BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &basis_binding)
        .map_err(|error| {
            denial(
                BridgeExecutionBasisDenialKind::SignalAttemptAdmissionFailed,
                error.detail().to_owned(),
                &counters,
            )
        })?;
    let request = runtime
        .admit_async_request_identity(request)
        .map_err(|error| {
            denial(
                BridgeExecutionBasisDenialKind::SignalAttemptAdmissionFailed,
                error.detail().to_owned(),
                &counters,
            )
        })?;
    counters.admitted_signal_attempt();
    counters.checked_signal_attempt();
    if let Err(denial) = validate_signal_attempt(runtime, &request, &counters) {
        cancel_failed_signal_admission(runtime, &request);
        return Err(denial);
    }
    let posture = match step_contract {
        Some(step_contract) => {
            let queue = bind_signal_managed_queue(
                runtime,
                &request,
                step_contract.queue_depth_ceiling(),
                &counters,
            )?;
            counters.bound_signal_queue();
            BridgeExecutionBasisPosture::Managed(BridgeManagedExecutionBasis {
                step_contract,
                queue,
                occupancy_width: 0,
            })
        }
        None => BridgeExecutionBasisPosture::Atomic,
    };

    Ok(BridgeBoundExecutionBasis::new(
        BridgeBoundExecutionBasisParts {
            bridge_runtime_key: runtime.signal_runtime_key,
            managed_intent: intent,
            posture,
            request,
            observation,
            authoritative_source_profile: runtime.authoritative_source_profile.clone(),
            reservation,
            counters,
        },
    ))
}

fn bind_signal_managed_queue(
    runtime: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
    queue_capacity: u64,
    counters: &BridgeExecutionBasisCounters,
) -> Result<ResourceManagedQueueBinding, BridgeExecutionBasisDenial> {
    let binding = with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime.bind_resource_managed_queue(request.signal_admission(), queue_capacity)
    })
    .map_err(|error| {
        denial(
            BridgeExecutionBasisDenialKind::SignalRuntimeThreadAffinityViolation,
            format!(
                "bridge Signal runtime {} belongs to thread {:?}, not {:?}",
                error.runtime_key(),
                error.owner(),
                error.current()
            ),
            counters,
        )
    })?;
    binding.map_err(|queue_denial| {
        cancel_failed_signal_admission(runtime, request);
        denial(
            BridgeExecutionBasisDenialKind::SignalManagedQueueBindingFailed,
            queue_denial.detail(),
            counters,
        )
    })
}

fn cancel_failed_signal_admission(
    runtime: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
) {
    let _ = with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime.cancel_resource_request(
            request.request_handle(),
            ResourceCancellationReason::RuntimePolicy,
        )
    });
}

fn validate_managed_intent(
    intent: &BridgeManagedExecutionIntent,
    counters: &BridgeExecutionBasisCounters,
) -> Result<(), BridgeExecutionBasisDenial> {
    if intent.operation_binding_identity().is_empty()
        || intent.resource_attempt_identity().is_empty()
    {
        return Err(denial(
            BridgeExecutionBasisDenialKind::InvalidManagedExecutionIntent,
            "bridge managed execution requires non-empty operation and resource-attempt identities",
            counters,
        ));
    }
    Ok(())
}

fn validate_signal_attempt(
    runtime: &RuntimeBridge,
    request: &AdmittedBridgeAsyncRequestIdentity,
    counters: &BridgeExecutionBasisCounters,
) -> Result<(), BridgeExecutionBasisDenial> {
    with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime
            .in_flight_resource_request(request.request_handle())
            .cloned()
    })
    .map_err(|error| {
        denial(
            BridgeExecutionBasisDenialKind::SignalRuntimeThreadAffinityViolation,
            format!(
                "bridge Signal runtime {} belongs to thread {:?}, not {:?}",
                error.runtime_key(),
                error.owner(),
                error.current()
            ),
            counters,
        )
    })?
    .ok_or_else(|| {
        denial(
            BridgeExecutionBasisDenialKind::SignalAttemptMissing,
            "bridge execution basis requires a live Signal resource request",
            counters,
        )
    })
    .and_then(|in_flight| {
        if in_flight.attempt() != request.attempt()
            || in_flight.request_intent_digest().as_str() != request.request_intent_digest()
        {
            return Err(denial(
                BridgeExecutionBasisDenialKind::SignalAttemptMismatch,
                "bridge execution basis Signal request attempt no longer matches admission",
                counters,
            ));
        }
        Ok(())
    })
}

fn validate_truth_basis(
    request: &BridgeAsyncRequestTruthViewBasis,
    planned: &PlannedTruthViewPacket,
    counters: &BridgeExecutionBasisCounters,
) -> Result<(), BridgeExecutionBasisDenial> {
    if request.kind() == BridgeAsyncRequestTruthViewBasisKind::Preview {
        return Err(denial(
            BridgeExecutionBasisDenialKind::PreviewBasisUnsupported,
            "preview subscription truth cannot authorize a managed domain execution basis",
            counters,
        ));
    }
    let planned_basis = planned.authority_basis();
    let branch_matches = request.truth_branch_identity() == Some(planned_basis.branch_identity());
    let snapshot_matches = request.truth_snapshot_identity() == planned_basis.snapshot_identity();
    let commit_matches = match request.kind() {
        BridgeAsyncRequestTruthViewBasisKind::BranchHead => true,
        _ => request.truth_commit_identity() == planned_basis.commit_identity(),
    };
    if !branch_matches || !snapshot_matches || !commit_matches {
        return Err(denial(
            BridgeExecutionBasisDenialKind::TruthBasisMismatch,
            "bridge async request and planned truth view do not share one exact basis",
            counters,
        ));
    }
    Ok(())
}

fn reserve_intent(
    runtime: &RuntimeBridge,
    intent: &BridgeManagedExecutionIntent,
    counters: &BridgeExecutionBasisCounters,
) -> Result<super::reservation::BridgeExecutionBasisReservation, BridgeExecutionBasisDenial> {
    runtime
        .execution_basis_reservations
        .reserve(BridgeExecutionBasisReservationKey::new(
            intent.identity().clone(),
        ))
        .ok_or_else(|| {
            denial(
                BridgeExecutionBasisDenialKind::ManagedExecutionIntentAlreadyReserved,
                "bridge managed execution intent already owns an active execution basis",
                counters,
            )
        })
}

fn denial(
    kind: BridgeExecutionBasisDenialKind,
    detail: impl Into<String>,
    counters: &BridgeExecutionBasisCounters,
) -> BridgeExecutionBasisDenial {
    BridgeExecutionBasisDenial::new(kind, detail, counters.clone())
}
