use worth_signal::facade::{ResourceCancellationReason, ResourceInFlightStatus};

use crate::execution_basis::reservation::BridgeExecutionBasisReservation;
use crate::execution_basis::{
    BridgeExecutionBasisFinalizationReceipt, BridgeExecutionBasisReadmissionPending,
    BridgeExecutionBasisReadmissionYielded, BridgeYieldedExecutionBasis,
};
use crate::source::{with_async_request_signal_runtime, AdmittedBridgeAsyncRequestIdentity};

use super::BridgeExecutionBasisReadmissionCounters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionBasisReadmissionDenialKind {
    ForeignRuntime,
    SourceProfileMismatch,
    OperationBindingMismatch,
    AttemptIdentityReused,
    InvalidManagedExecutionIntent,
    ManagedExecutionIntentAlreadyReserved,
    SignalDeclarationUnavailable,
    SignalAttemptAdmissionFailed,
    SignalAttemptMissing,
    SignalAttemptMismatch,
    SignalManagedQueueBindingFailed,
    YieldPostureMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionBasisReadmissionRecoveryKind {
    ProvisionalSignalCleanupFailed,
}

#[must_use = "readmission outcomes retain live or recoverable execution-basis authority"]
pub enum BridgeExecutionBasisReadmissionOutcome {
    Pending(BridgeExecutionBasisReadmissionPending),
    Denied(BridgeExecutionBasisReadmissionDenied),
    RecoveryRequired(BridgeExecutionBasisReadmissionRecoveryRequired),
}

#[must_use = "readmission denial retains the yielded execution-basis authority"]
pub struct BridgeExecutionBasisReadmissionDenied {
    kind: BridgeExecutionBasisReadmissionDenialKind,
    detail: String,
    yielded: BridgeYieldedExecutionBasis,
    counters: BridgeExecutionBasisReadmissionCounters,
}

#[must_use = "readmission recovery must be resolved to release provisional Signal authority"]
pub struct BridgeExecutionBasisReadmissionRecoveryRequired {
    detail: String,
    yielded: Option<BridgeYieldedExecutionBasis>,
    provisional: Option<BridgeProvisionalSignalAttempt>,
    counters: BridgeExecutionBasisReadmissionCounters,
}

#[must_use = "cleanup outcomes retain yielded or recovery authority"]
pub enum BridgeExecutionBasisReadmissionCleanupOutcome {
    Complete(BridgeExecutionBasisReadmissionYielded),
    RecoveryRequired(BridgeExecutionBasisReadmissionRecoveryRequired),
}

pub(super) struct BridgeProvisionalSignalAttempt {
    runtime_key: u64,
    request: Option<AdmittedBridgeAsyncRequestIdentity>,
    reservation: Option<BridgeExecutionBasisReservation>,
}

impl BridgeExecutionBasisReadmissionDenied {
    pub(super) fn new(
        kind: BridgeExecutionBasisReadmissionDenialKind,
        detail: impl Into<String>,
        yielded: BridgeYieldedExecutionBasis,
        counters: BridgeExecutionBasisReadmissionCounters,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            yielded,
            counters,
        }
    }

    pub const fn kind(&self) -> BridgeExecutionBasisReadmissionDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(&self) -> BridgeExecutionBasisReadmissionCounters {
        self.counters
    }

    pub fn yielded_receipt(&self) -> &BridgeExecutionBasisFinalizationReceipt {
        self.yielded.receipt()
    }

    pub fn into_returned_yielded(self) -> BridgeExecutionBasisReadmissionYielded {
        BridgeExecutionBasisReadmissionYielded::new(self.yielded, self.counters)
    }
}

impl BridgeExecutionBasisReadmissionRecoveryRequired {
    pub(super) fn new(
        detail: impl Into<String>,
        yielded: BridgeYieldedExecutionBasis,
        provisional: BridgeProvisionalSignalAttempt,
        counters: BridgeExecutionBasisReadmissionCounters,
    ) -> Self {
        Self {
            detail: detail.into(),
            yielded: Some(yielded),
            provisional: Some(provisional),
            counters,
        }
    }

    pub const fn kind(&self) -> BridgeExecutionBasisReadmissionRecoveryKind {
        BridgeExecutionBasisReadmissionRecoveryKind::ProvisionalSignalCleanupFailed
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(&self) -> BridgeExecutionBasisReadmissionCounters {
        self.counters
    }

    pub fn yielded_receipt(&self) -> &BridgeExecutionBasisFinalizationReceipt {
        self.yielded
            .as_ref()
            .expect("readmission recovery retains yielded authority")
            .receipt()
    }

    pub fn retry_cleanup(mut self) -> BridgeExecutionBasisReadmissionCleanupOutcome {
        let yielded = self
            .yielded
            .take()
            .expect("readmission recovery cleanup consumes yielded authority once");
        let mut provisional = self
            .provisional
            .take()
            .expect("readmission recovery cleanup consumes provisional authority once");
        match provisional.cleanup() {
            Ok(()) => BridgeExecutionBasisReadmissionCleanupOutcome::Complete(
                BridgeExecutionBasisReadmissionYielded::new(yielded, self.counters),
            ),
            Err(detail) => BridgeExecutionBasisReadmissionCleanupOutcome::RecoveryRequired(
                Self::new(detail, yielded, provisional, self.counters),
            ),
        }
    }
}

impl BridgeProvisionalSignalAttempt {
    pub(super) fn new(
        runtime_key: u64,
        request: AdmittedBridgeAsyncRequestIdentity,
        reservation: BridgeExecutionBasisReservation,
    ) -> Self {
        Self {
            runtime_key,
            request: Some(request),
            reservation: Some(reservation),
        }
    }

    pub(super) fn request(&self) -> &AdmittedBridgeAsyncRequestIdentity {
        self.request
            .as_ref()
            .expect("provisional Signal authority retains its request")
    }

    pub(super) fn request_identity(&self) -> &str {
        self.request().digest()
    }

    pub(super) fn cleanup(&mut self) -> Result<(), String> {
        cancel_provisional_request(self.runtime_key, self.request())?;
        self.request.take();
        self.reservation.take();
        Ok(())
    }

    pub(super) fn into_parts(
        mut self,
    ) -> (
        AdmittedBridgeAsyncRequestIdentity,
        BridgeExecutionBasisReservation,
    ) {
        (
            self.request
                .take()
                .expect("pending readmission retains its Signal request"),
            self.reservation
                .take()
                .expect("pending readmission retains its Bridge reservation"),
        )
    }
}

impl Drop for BridgeProvisionalSignalAttempt {
    fn drop(&mut self) {
        let Some(request) = self.request.as_ref() else {
            return;
        };
        let _ = cancel_provisional_request(self.runtime_key, request);
    }
}

fn cancel_provisional_request(
    runtime_key: u64,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> Result<(), String> {
    with_async_request_signal_runtime(runtime_key, |runtime| {
        if runtime
            .in_flight_resource_request(request.request_handle())
            .is_some_and(|in_flight| in_flight.status() != ResourceInFlightStatus::Active)
        {
            return Ok(());
        }
        let report = runtime
            .cancel_resource_request(
                request.request_handle(),
                ResourceCancellationReason::RuntimePolicy,
            )
            .map_err(|error| error.to_string())?;
        report
            .cancelled_request()
            .map(|_| ())
            .ok_or_else(|| "Signal denied provisional readmission cancellation".to_owned())
    })
    .map_err(|error| {
        format!(
            "bridge Signal runtime {} belongs to thread {:?}, not {:?}",
            error.runtime_key(),
            error.owner(),
            error.current()
        )
    })?
}

mod denial_report;
