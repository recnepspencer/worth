use worth_signal::facade::{
    RawCompletionEnvelope, ResourceCancellationReason, ResourceInFlightStatus,
    ResourceLifecycleClass,
};

use crate::source::with_async_request_signal_runtime;

use super::{
    BridgeBoundExecutionBasis, BridgeExecutionBasisIdentity, BridgeManagedExecutionIntentIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionBasisTerminalDisposition {
    Completed,
    Yielded,
    Cancelled,
    Abandoned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionBasisSignalTerminal {
    Fulfilled,
    Cancelled,
    TimedOut,
    Rejected,
    Superseded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionBasisFinalizationFailureKind {
    AtomicExecutionUnsupported,
    ManagedQueueOccupied,
    SignalRuntimeThreadAffinityViolation,
    SignalCompletionDenied,
    SignalCompletionStagingFailed,
    SignalCompletionCommitFailed,
    SignalCancellationDenied,
    SignalCancellationFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeExecutionBasisFinalizationReceipt {
    basis_identity: BridgeExecutionBasisIdentity,
    intent_identity: BridgeManagedExecutionIntentIdentity,
    signal_terminal: BridgeExecutionBasisSignalTerminal,
    disposition: BridgeExecutionBasisTerminalDisposition,
    signal_transition_performed: bool,
    reservation_released: bool,
}

pub struct BridgeExecutionBasisFinalizationFailure {
    kind: BridgeExecutionBasisFinalizationFailureKind,
    detail: String,
    basis: BridgeBoundExecutionBasis,
}

impl BridgeBoundExecutionBasis {
    pub fn finalize(
        mut self,
        disposition: BridgeExecutionBasisTerminalDisposition,
    ) -> Result<BridgeExecutionBasisFinalizationReceipt, BridgeExecutionBasisFinalizationFailure>
    {
        if disposition == BridgeExecutionBasisTerminalDisposition::Yielded
            && self.step_contract().is_none()
        {
            return Err(BridgeExecutionBasisFinalizationFailure::new(
                BridgeExecutionBasisFinalizationFailureKind::AtomicExecutionUnsupported,
                "atomic execution cannot yield",
                self,
            ));
        }
        let occupancy_width = self
            .posture
            .managed()
            .map_or(0, |managed| managed.occupancy_width);
        if occupancy_width != 0 {
            return Err(BridgeExecutionBasisFinalizationFailure {
                kind: BridgeExecutionBasisFinalizationFailureKind::ManagedQueueOccupied,
                detail: format!(
                    "managed execution basis retains {} units of Signal queue occupancy",
                    occupancy_width
                ),
                basis: self,
            });
        }
        let (signal_terminal, signal_transition_performed) =
            match finalize_signal_request(&self, disposition) {
                Ok(finalization) => finalization,
                Err((kind, detail)) => {
                    return Err(BridgeExecutionBasisFinalizationFailure {
                        kind,
                        detail,
                        basis: self,
                    });
                }
            };
        self.signal_terminalized = true;
        let reservation_released = self
            .reservation
            .take()
            .is_some_and(super::reservation::BridgeExecutionBasisReservation::release);
        Ok(BridgeExecutionBasisFinalizationReceipt {
            basis_identity: self.identity.clone(),
            intent_identity: self.managed_intent.identity().clone(),
            signal_terminal,
            disposition,
            signal_transition_performed,
            reservation_released,
        })
    }
}

impl BridgeExecutionBasisFinalizationReceipt {
    pub(super) fn new(
        basis_identity: BridgeExecutionBasisIdentity,
        intent_identity: BridgeManagedExecutionIntentIdentity,
        signal_terminal: BridgeExecutionBasisSignalTerminal,
        disposition: BridgeExecutionBasisTerminalDisposition,
        signal_transition_performed: bool,
        reservation_released: bool,
    ) -> Self {
        Self {
            basis_identity,
            intent_identity,
            signal_terminal,
            disposition,
            signal_transition_performed,
            reservation_released,
        }
    }

    pub fn basis_identity(&self) -> &BridgeExecutionBasisIdentity {
        &self.basis_identity
    }

    pub fn intent_identity(&self) -> &BridgeManagedExecutionIntentIdentity {
        &self.intent_identity
    }

    pub fn signal_terminal(&self) -> BridgeExecutionBasisSignalTerminal {
        self.signal_terminal
    }

    pub fn disposition(&self) -> BridgeExecutionBasisTerminalDisposition {
        self.disposition
    }

    pub fn signal_transition_performed(&self) -> bool {
        self.signal_transition_performed
    }

    pub fn reservation_released(&self) -> bool {
        self.reservation_released
    }
}

impl BridgeExecutionBasisFinalizationFailure {
    pub(super) fn new(
        kind: BridgeExecutionBasisFinalizationFailureKind,
        detail: impl Into<String>,
        basis: BridgeBoundExecutionBasis,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            basis,
        }
    }

    pub fn kind(&self) -> BridgeExecutionBasisFinalizationFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn into_basis(self) -> BridgeBoundExecutionBasis {
        self.basis
    }
}

impl std::fmt::Debug for BridgeExecutionBasisFinalizationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeExecutionBasisFinalizationFailure")
            .field("kind", &self.kind)
            .field("detail", &self.detail)
            .field("basis_identity", &self.basis.identity())
            .finish()
    }
}

pub(super) fn finalize_signal_request(
    basis: &BridgeBoundExecutionBasis,
    disposition: BridgeExecutionBasisTerminalDisposition,
) -> Result<
    (BridgeExecutionBasisSignalTerminal, bool),
    (BridgeExecutionBasisFinalizationFailureKind, String),
> {
    with_async_request_signal_runtime(basis.bridge_runtime_key, |runtime| {
        if let Some(terminal) = existing_signal_terminal(runtime, basis) {
            return Ok((terminal, false));
        }
        let terminal = match disposition {
            BridgeExecutionBasisTerminalDisposition::Completed => {
                complete_signal_request(runtime, basis)
            }
            BridgeExecutionBasisTerminalDisposition::Yielded => {
                cancel_signal_request(runtime, basis, ResourceCancellationReason::RuntimePolicy)
            }
            BridgeExecutionBasisTerminalDisposition::Cancelled => {
                cancel_signal_request(runtime, basis, ResourceCancellationReason::HostRequested)
            }
            BridgeExecutionBasisTerminalDisposition::Abandoned => {
                cancel_signal_request(runtime, basis, ResourceCancellationReason::RuntimePolicy)
            }
        }?;
        Ok((terminal, true))
    })
    .map_err(|error| {
        (
            BridgeExecutionBasisFinalizationFailureKind::SignalRuntimeThreadAffinityViolation,
            format!(
                "bridge Signal runtime {} belongs to thread {:?}, not {:?}",
                error.runtime_key(),
                error.owner(),
                error.current()
            ),
        )
    })?
}

fn existing_signal_terminal(
    runtime: &mut crate::source::BridgeSignalRuntime,
    basis: &BridgeBoundExecutionBasis,
) -> Option<BridgeExecutionBasisSignalTerminal> {
    let status = runtime
        .in_flight_resource_request(basis.request.request_handle())?
        .status();
    match status {
        ResourceInFlightStatus::Active => None,
        ResourceInFlightStatus::Fulfilled => Some(BridgeExecutionBasisSignalTerminal::Fulfilled),
        ResourceInFlightStatus::Cancelled => Some(BridgeExecutionBasisSignalTerminal::Cancelled),
        ResourceInFlightStatus::TimedOut => Some(BridgeExecutionBasisSignalTerminal::TimedOut),
        ResourceInFlightStatus::Rejected => Some(BridgeExecutionBasisSignalTerminal::Rejected),
        ResourceInFlightStatus::Superseded => Some(BridgeExecutionBasisSignalTerminal::Superseded),
    }
}

fn complete_signal_request(
    runtime: &mut crate::source::BridgeSignalRuntime,
    basis: &BridgeBoundExecutionBasis,
) -> Result<BridgeExecutionBasisSignalTerminal, (BridgeExecutionBasisFinalizationFailureKind, String)>
{
    let request = &basis.request;
    let descriptor = request
        .lowered()
        .resource_descriptor()
        .expect("managed execution declaration is request-response");
    let raw = RawCompletionEnvelope::new(
        request.request_handle().request_id(),
        request.request_handle().generation(),
        request.request_handle().branch_epoch(),
        request.attempt(),
        descriptor.payload_contract_digest().clone(),
        0,
    );
    let admitted = runtime
        .admit_resource_completion(raw)
        .admitted_completion()
        .ok_or_else(|| {
            (
                BridgeExecutionBasisFinalizationFailureKind::SignalCompletionDenied,
                "Signal denied completion of the managed execution request".to_owned(),
            )
        })?;
    let staged = runtime
        .stage_admitted_resource_completion(admitted)
        .map_err(|error| {
            (
                BridgeExecutionBasisFinalizationFailureKind::SignalCompletionStagingFailed,
                error.to_string(),
            )
        })?
        .staged_effect();
    let committed = runtime
        .commit_staged_resource_completion(staged)
        .map_err(|error| {
            (
                BridgeExecutionBasisFinalizationFailureKind::SignalCompletionCommitFailed,
                error.to_string(),
            )
        })?;
    if committed.lifecycle().lifecycle() != ResourceLifecycleClass::Fulfilled {
        return Err((
            BridgeExecutionBasisFinalizationFailureKind::SignalCompletionCommitFailed,
            "Signal completion committed without a fulfilled lifecycle".to_owned(),
        ));
    }
    Ok(BridgeExecutionBasisSignalTerminal::Fulfilled)
}

fn cancel_signal_request(
    runtime: &mut crate::source::BridgeSignalRuntime,
    basis: &BridgeBoundExecutionBasis,
    reason: ResourceCancellationReason,
) -> Result<BridgeExecutionBasisSignalTerminal, (BridgeExecutionBasisFinalizationFailureKind, String)>
{
    let report = runtime
        .cancel_resource_request(basis.request.request_handle(), reason)
        .map_err(|error| {
            (
                BridgeExecutionBasisFinalizationFailureKind::SignalCancellationFailed,
                error.to_string(),
            )
        })?;
    report.cancelled_request().ok_or_else(|| {
        (
            BridgeExecutionBasisFinalizationFailureKind::SignalCancellationDenied,
            "Signal denied cancellation of the managed execution request".to_owned(),
        )
    })?;
    Ok(BridgeExecutionBasisSignalTerminal::Cancelled)
}
