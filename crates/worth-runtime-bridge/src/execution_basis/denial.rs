use super::BridgeExecutionBasisCounters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionBasisDenialKind {
    ExecutionDenied(crate::error::BridgeExecutionDenial),
    InvalidManagedExecutionIntent,
    SignalRuntimeThreadAffinityViolation,
    SignalDeclarationUnavailable,
    SignalAttemptAdmissionFailed,
    SignalAttemptMissing,
    SignalAttemptMismatch,
    SignalManagedQueueBindingFailed,
    TruthBasisMismatch,
    PreviewBasisUnsupported,
    ManagedExecutionIntentAlreadyReserved,
    TruthMaterializationFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeExecutionBasisDenial {
    kind: BridgeExecutionBasisDenialKind,
    detail: String,
    counters: BridgeExecutionBasisCounters,
}

impl BridgeExecutionBasisDenial {
    pub(crate) fn new(
        kind: BridgeExecutionBasisDenialKind,
        detail: impl Into<String>,
        counters: BridgeExecutionBasisCounters,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            counters,
        }
    }

    pub fn kind(&self) -> BridgeExecutionBasisDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn counters(&self) -> &BridgeExecutionBasisCounters {
        &self.counters
    }
}

pub(super) fn truth_materialization_denial(
    error: &crate::error::BridgeDeliveryError,
) -> BridgeExecutionBasisDenialKind {
    use crate::error::BridgeDeliveryErrorKind as Delivery;
    let execution = match error.kind() {
        Delivery::ExecutionDenied(cause) => Some(cause),
        Delivery::SnapshotAcquisitionFailure(source)
        | Delivery::HistoricalTruthViewUnavailable(source) => match source.kind() {
            crate::adapter::RelationalBridgeSourceErrorTag::ExecutionDenied(cause) => Some(cause),
            _ => None,
        },
        Delivery::SnapshotReadFailure(read) | Delivery::SnapshotReadContractViolation(read) => {
            match read.kind() {
                crate::snapshot::BridgeSnapshotReadErrorKind::ExecutionDenied(cause) => Some(cause),
                _ => None,
            }
        }
        _ => None,
    };
    execution.map_or(
        BridgeExecutionBasisDenialKind::TruthMaterializationFailed,
        BridgeExecutionBasisDenialKind::ExecutionDenied,
    )
}

#[cfg(test)]
mod read_stop_tests;
