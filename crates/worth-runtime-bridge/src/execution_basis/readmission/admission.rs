use worth_signal::facade::{ResourceInFlightStatus, ResourceManagedQueueBinding};

use crate::execution_basis::reservation::BridgeExecutionBasisReservationKey;
use crate::execution_basis::{
    BridgeExecutionBasisCounters, BridgeManagedExecutionIntent, BridgeYieldedExecutionBasis,
};
use crate::facade::RuntimeBridge;
use crate::source::{
    with_async_request_signal_runtime, BridgeAsyncRequestAdmissionRequest,
    ValidatedBridgeAsyncRequestBasisBinding,
};

use super::outcome::{
    BridgeExecutionBasisReadmissionDenialKind, BridgeExecutionBasisReadmissionDenied,
    BridgeExecutionBasisReadmissionOutcome, BridgeExecutionBasisReadmissionRecoveryRequired,
    BridgeProvisionalSignalAttempt,
};
use super::{
    BridgeExecutionBasisReadmissionCounters, BridgeExecutionBasisReadmissionPending,
    BridgeYieldedExecutionBasisPreflight,
};

pub(crate) fn readmit_yielded_execution_basis(
    runtime: &RuntimeBridge,
    preflight: BridgeYieldedExecutionBasisPreflight,
    fresh_intent: BridgeManagedExecutionIntent,
) -> BridgeExecutionBasisReadmissionOutcome {
    let (yielded, operation_binding_identity, mut counters) = preflight.into_parts();
    if let Some((kind, detail)) =
        fresh_intent_denial(&yielded, &operation_binding_identity, &fresh_intent)
    {
        return denied(kind, detail, yielded, counters);
    }

    counters.checked_reservation();
    let Some(reservation) =
        runtime
            .execution_basis_reservations
            .reserve(BridgeExecutionBasisReservationKey::new(
                fresh_intent.identity().clone(),
            ))
    else {
        return denied(
            BridgeExecutionBasisReadmissionDenialKind::ManagedExecutionIntentAlreadyReserved,
            "fresh Bridge managed intent is already reserved",
            yielded,
            counters,
        );
    };

    let mut basis_counters = BridgeExecutionBasisCounters::default();
    basis_counters.checked_managed_intent();
    basis_counters.checked_truth_basis();
    basis_counters.checked_reservation();
    let lowered = match yielded
        .basis
        .request
        .lowered()
        .instantiate_request_response(fresh_intent.identity().as_str())
    {
        Ok(lowered) => lowered,
        Err(error) => {
            return denied(
                BridgeExecutionBasisReadmissionDenialKind::SignalDeclarationUnavailable,
                error.detail(),
                yielded,
                counters,
            );
        }
    };
    let truth_basis = yielded
        .basis
        .request
        .basis_binding()
        .truth_view_basis()
        .clone();
    let basis_binding = ValidatedBridgeAsyncRequestBasisBinding::bind(&lowered, truth_basis);
    let request =
        match BridgeAsyncRequestAdmissionRequest::request_response(&lowered, &basis_binding) {
            Ok(request) => request,
            Err(error) => {
                return denied(
                    BridgeExecutionBasisReadmissionDenialKind::SignalAttemptAdmissionFailed,
                    error.detail(),
                    yielded,
                    counters,
                );
            }
        };
    let request = match runtime.admit_async_request_identity(request) {
        Ok(request) => request,
        Err(error) => {
            return denied(
                BridgeExecutionBasisReadmissionDenialKind::SignalAttemptAdmissionFailed,
                error.detail(),
                yielded,
                counters,
            );
        }
    };
    counters.admitted_signal_attempt();
    basis_counters.admitted_signal_attempt();
    let provisional =
        BridgeProvisionalSignalAttempt::new(runtime.signal_runtime_key, request, reservation);

    counters.checked_signal_attempt();
    basis_counters.checked_signal_attempt();
    if let Err((kind, detail)) = validate_signal_attempt(runtime, &provisional) {
        return deny_after_signal_cleanup(kind, detail, yielded, provisional, counters);
    }
    let managed_queue = match bind_managed_queue(runtime, &provisional, yielded.step_contract()) {
        Ok(queue) => queue,
        Err((kind, detail)) => {
            return deny_after_signal_cleanup(kind, detail, yielded, provisional, counters);
        }
    };
    counters.bound_signal_queue();
    basis_counters.bound_signal_queue();
    let (request, reservation) = provisional.into_parts();
    BridgeExecutionBasisReadmissionOutcome::Pending(BridgeExecutionBasisReadmissionPending::new(
        yielded,
        runtime.signal_runtime_key,
        fresh_intent,
        managed_queue,
        request,
        reservation,
        basis_counters,
        counters,
    ))
}

fn fresh_intent_denial(
    yielded: &BridgeYieldedExecutionBasis,
    operation_binding_identity: &str,
    fresh_intent: &BridgeManagedExecutionIntent,
) -> Option<(BridgeExecutionBasisReadmissionDenialKind, &'static str)> {
    if fresh_intent.operation_binding_identity() != operation_binding_identity {
        return Some((
            BridgeExecutionBasisReadmissionDenialKind::OperationBindingMismatch,
            "fresh Bridge intent names a different operation binding",
        ));
    }
    if fresh_intent.resource_attempt_identity()
        == yielded.managed_intent().resource_attempt_identity()
    {
        return Some((
            BridgeExecutionBasisReadmissionDenialKind::AttemptIdentityReused,
            "Bridge readmission requires a fresh Query resource-attempt identity",
        ));
    }
    if fresh_intent.operation_binding_identity().is_empty()
        || fresh_intent.resource_attempt_identity().is_empty()
    {
        return Some((
            BridgeExecutionBasisReadmissionDenialKind::InvalidManagedExecutionIntent,
            "Bridge readmission requires non-empty operation and attempt identities",
        ));
    }
    None
}

fn validate_signal_attempt(
    runtime: &RuntimeBridge,
    provisional: &BridgeProvisionalSignalAttempt,
) -> Result<(), (BridgeExecutionBasisReadmissionDenialKind, String)> {
    let request = provisional.request();
    let in_flight =
        with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
            signal_runtime
                .in_flight_resource_request(request.request_handle())
                .cloned()
        })
        .map_err(|error| {
            (
                BridgeExecutionBasisReadmissionDenialKind::SignalAttemptMissing,
                format!(
                    "bridge Signal runtime {} belongs to thread {:?}, not {:?}",
                    error.runtime_key(),
                    error.owner(),
                    error.current()
                ),
            )
        })?
        .ok_or_else(|| {
            (
                BridgeExecutionBasisReadmissionDenialKind::SignalAttemptMissing,
                "fresh Signal attempt disappeared during Bridge readmission".to_owned(),
            )
        })?;
    if in_flight.status() != ResourceInFlightStatus::Active
        || in_flight.attempt() != request.attempt()
        || in_flight.request_intent_digest().as_str() != request.request_intent_digest()
    {
        return Err((
            BridgeExecutionBasisReadmissionDenialKind::SignalAttemptMismatch,
            "fresh Signal attempt does not match the Bridge admission".to_owned(),
        ));
    }
    Ok(())
}

fn bind_managed_queue(
    runtime: &RuntimeBridge,
    provisional: &BridgeProvisionalSignalAttempt,
    step_contract: &crate::execution_basis::BridgeManagedExecutionStepContract,
) -> Result<ResourceManagedQueueBinding, (BridgeExecutionBasisReadmissionDenialKind, String)> {
    with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime.bind_resource_managed_queue(
            provisional.request().signal_admission(),
            step_contract.queue_depth_ceiling(),
        )
    })
    .map_err(|error| {
        (
            BridgeExecutionBasisReadmissionDenialKind::SignalManagedQueueBindingFailed,
            format!(
                "bridge Signal runtime {} belongs to thread {:?}, not {:?}",
                error.runtime_key(),
                error.owner(),
                error.current()
            ),
        )
    })?
    .map_err(|denial| {
        (
            BridgeExecutionBasisReadmissionDenialKind::SignalManagedQueueBindingFailed,
            denial.detail().to_owned(),
        )
    })
}

fn deny_after_signal_cleanup(
    kind: BridgeExecutionBasisReadmissionDenialKind,
    detail: String,
    yielded: BridgeYieldedExecutionBasis,
    mut provisional: BridgeProvisionalSignalAttempt,
    counters: BridgeExecutionBasisReadmissionCounters,
) -> BridgeExecutionBasisReadmissionOutcome {
    match provisional.cleanup() {
        Ok(()) => denied(kind, detail, yielded, counters),
        Err(cleanup_detail) => BridgeExecutionBasisReadmissionOutcome::RecoveryRequired(
            BridgeExecutionBasisReadmissionRecoveryRequired::new(
                format!("{detail}; cleanup failed: {cleanup_detail}"),
                yielded,
                provisional,
                counters,
            ),
        ),
    }
}

fn denied(
    kind: BridgeExecutionBasisReadmissionDenialKind,
    detail: impl Into<String>,
    yielded: BridgeYieldedExecutionBasis,
    counters: BridgeExecutionBasisReadmissionCounters,
) -> BridgeExecutionBasisReadmissionOutcome {
    BridgeExecutionBasisReadmissionOutcome::Denied(BridgeExecutionBasisReadmissionDenied::new(
        kind, detail, yielded, counters,
    ))
}
