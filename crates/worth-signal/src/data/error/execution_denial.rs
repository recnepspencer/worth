//! The exhaustive translation door for execution authority and checkpoints.

use worth_execution::{LeaseDenial, MapKernelFailure, MapKernelStop, MapStop, MemoryLimitDenial};

use super::{SignalError, SignalExecutionFailure, SignalExecutionStopReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalLeaseDenial {
    WorkerLimitExceedsParent,
    MemoryLimitExceedsParent,
    WorkLimitExceedsParent,
    MemoryExhausted(MemoryLimitDenial),
    ChargedBytesOverflow,
    UnrelatedNestedLease,
    NoActiveExecutionScope,
    EquivalenceContractUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalCheckpointDenial {
    Cancelled,
    DeadlineElapsed,
    WorkCounterOverflow,
    WorkCeiling,
    NestedStopped,
}

impl From<LeaseDenial> for SignalLeaseDenial {
    fn from(denial: LeaseDenial) -> Self {
        match denial {
            LeaseDenial::WorkerLimitExceedsParent => Self::WorkerLimitExceedsParent,
            LeaseDenial::MemoryLimitExceedsParent => Self::MemoryLimitExceedsParent,
            LeaseDenial::WorkLimitExceedsParent => Self::WorkLimitExceedsParent,
            LeaseDenial::MemoryExhausted(denial) => Self::MemoryExhausted(denial),
            LeaseDenial::ChargedBytesOverflow => Self::ChargedBytesOverflow,
            LeaseDenial::UnrelatedNestedLease => Self::UnrelatedNestedLease,
            LeaseDenial::NoActiveExecutionScope => Self::NoActiveExecutionScope,
            LeaseDenial::EquivalenceContractUnavailable => Self::EquivalenceContractUnavailable,
        }
    }
}

impl From<MapKernelStop> for SignalCheckpointDenial {
    fn from(stop: MapKernelStop) -> Self {
        match stop {
            MapKernelStop::Cancelled => Self::Cancelled,
            MapKernelStop::DeadlineElapsed => Self::DeadlineElapsed,
            MapKernelStop::WorkCounterOverflow => Self::WorkCounterOverflow,
            MapKernelStop::WorkCeiling => Self::WorkCeiling,
            MapKernelStop::NestedStopped => Self::NestedStopped,
        }
    }
}

impl From<SignalCheckpointDenial> for SignalExecutionFailure {
    fn from(stop: SignalCheckpointDenial) -> Self {
        match stop {
            SignalCheckpointDenial::Cancelled => Self::Cancelled,
            SignalCheckpointDenial::DeadlineElapsed => Self::DeadlineElapsed,
            SignalCheckpointDenial::WorkCounterOverflow => Self::WorkCounterOverflow,
            SignalCheckpointDenial::WorkCeiling => Self::WorkCeiling,
            SignalCheckpointDenial::NestedStopped => Self::NestedStopped,
        }
    }
}

impl SignalError {
    pub fn execution_scope_denied(denial: worth_execution::WorkCeilingDenial) -> Self {
        match denial {
            worth_execution::WorkCeilingDenial::Admission(denial) => {
                Self::execution_admission_denied(denial)
            }
            worth_execution::WorkCeilingDenial::Stopped(stop) => {
                Self::execution_checkpoint_stopped(stop)
            }
            worth_execution::WorkCeilingDenial::Panicked => Self::ExecutionPanicked,
        }
    }

    pub fn execution_admission_denied(denial: LeaseDenial) -> Self {
        Self::ExecutionAdmissionDenied(denial.into())
    }

    pub fn execution_checkpoint_stopped(stop: MapKernelStop) -> Self {
        Self::ExecutionCheckpointStopped(stop.into())
    }
}

impl SignalError {
    pub(crate) fn retained_storage_denied(
        denial: crate::data::retained_storage::RetainedStoragePreparationDenial,
    ) -> Self {
        use crate::data::retained_storage::RetainedStoragePreparationDenial as Denial;
        match denial {
            Denial::ExecutionStopped(stop) => Self::ExecutionCheckpointStopped(stop),
            Denial::WorkExhausted { maximum_visits } => {
                Self::ConditionalEvaluationWorkExhausted { maximum_visits }
            }
            Denial::ChargeOverflow => Self::RetainedStorageChargeOverflow,
            Denial::ChargeUnderflow => Self::RetainedStorageChargeUnderflow,
            Denial::RetainedExtentHistoryUnavailable => Self::RetainedStorageHistoryUnavailable,
        }
    }
}

impl SignalError {
    pub(crate) fn request_scan_stopped(
        reason: worth_execution::MapStop<SignalError>,
        boundary: Option<worth_foundational::PartitionIdentity>,
        progress: super::SignalPublicationProgress,
        report: worth_foundational::ExecutionReport,
    ) -> Self {
        match reason {
            reason @ MapStop::Failure {
                cause:
                    MapKernelFailure::Domain(
                        SignalError::ExecutionCheckpointStopped(_)
                        | SignalError::ExecutionAdmissionDenied(_)
                        | SignalError::PreparationMemoryExhausted { .. }
                        | SignalError::ExecutionStopped(_),
                    ),
                ..
            } => Self::execution_stopped(super::SignalExecutionStop::new(
                reason.into(),
                boundary,
                progress,
                report,
            )),
            worth_execution::MapStop::Failure {
                cause: worth_execution::MapKernelFailure::Domain(error),
                ..
            } => error,
            reason @ (MapStop::Failure { .. }
            | MapStop::WorkExhausted { .. }
            | MapStop::Admission(_)) => Self::execution_stopped(super::SignalExecutionStop::new(
                reason.into(),
                boundary,
                progress,
                report,
            )),
        }
    }
}

impl From<MapStop<SignalError>> for SignalExecutionStopReason {
    fn from(stop: MapStop<SignalError>) -> Self {
        match stop {
            MapStop::Failure {
                identity,
                cause: MapKernelFailure::Domain(SignalError::ExecutionCheckpointStopped(stop)),
            } => Self::Failure {
                identity,
                cause: stop.into(),
            },
            MapStop::Failure {
                cause: MapKernelFailure::Domain(SignalError::ExecutionAdmissionDenied(denial)),
                ..
            } => Self::Admission(denial),
            MapStop::Failure {
                identity,
                cause:
                    MapKernelFailure::Domain(SignalError::PreparationMemoryExhausted {
                        required,
                        reserved,
                    }),
            } => Self::PreparationMemoryExhausted {
                identity,
                required,
                reserved,
            },
            MapStop::Failure { identity, cause } => Self::Failure {
                identity,
                cause: cause.into(),
            },
            MapStop::WorkExhausted { identity } => Self::WorkExhausted { identity },
            MapStop::Admission(denial) => Self::Admission(denial.into()),
        }
    }
}

impl From<MapKernelFailure<SignalError>> for SignalExecutionFailure {
    fn from(failure: MapKernelFailure<SignalError>) -> Self {
        match failure {
            MapKernelFailure::Domain(error) => Self::Domain(Box::new(error)),
            MapKernelFailure::Stop(stop) => SignalCheckpointDenial::from(stop).into(),
            MapKernelFailure::Panic => Self::Panic,
            MapKernelFailure::ResultCapacityExceeded => Self::ResultCapacityExceeded,
        }
    }
}

#[cfg(test)]
mod tests;

impl SignalError {
    /// The scan owns real checkpoint refusals. A nested failure instead carries
    /// its precise inner cause through the request's result slot.
    pub(crate) fn request_scan_failed(
        error: SignalError,
        stopped: Option<(
            MapStop<SignalError>,
            Option<worth_foundational::PartitionIdentity>,
        )>,
        identity: worth_foundational::PartitionIdentity,
        progress: super::SignalPublicationProgress,
        report: worth_foundational::ExecutionReport,
    ) -> Self {
        let carried = match stopped.as_ref() {
            None => true,
            Some((
                MapStop::Failure {
                    cause: MapKernelFailure::Stop(MapKernelStop::NestedStopped),
                    ..
                },
                _,
            )) => true,
            Some((
                MapStop::Failure {
                    cause:
                        MapKernelFailure::Stop(
                            MapKernelStop::Cancelled
                            | MapKernelStop::DeadlineElapsed
                            | MapKernelStop::WorkCounterOverflow
                            | MapKernelStop::WorkCeiling,
                        )
                        | MapKernelFailure::Domain(_)
                        | MapKernelFailure::Panic
                        | MapKernelFailure::ResultCapacityExceeded,
                    ..
                }
                | MapStop::WorkExhausted { .. }
                | MapStop::Admission(_),
                _,
            )) => false,
        };
        let (reason, boundary) = if carried {
            (
                MapStop::Failure {
                    identity,
                    cause: MapKernelFailure::Domain(error),
                },
                Some(identity),
            )
        } else {
            stopped.expect("a real scan stop takes precedence")
        };
        Self::request_scan_stopped(reason, boundary, progress, report)
    }
}
