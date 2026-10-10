use super::*;

#[test]
fn checkpoint_door_keeps_every_cause_distinct() {
    for (stop, expected) in [
        (MapKernelStop::Cancelled, SignalCheckpointDenial::Cancelled),
        (
            MapKernelStop::DeadlineElapsed,
            SignalCheckpointDenial::DeadlineElapsed,
        ),
        (
            MapKernelStop::WorkCounterOverflow,
            SignalCheckpointDenial::WorkCounterOverflow,
        ),
        (
            MapKernelStop::WorkCeiling,
            SignalCheckpointDenial::WorkCeiling,
        ),
        (
            MapKernelStop::NestedStopped,
            SignalCheckpointDenial::NestedStopped,
        ),
    ] {
        assert_eq!(
            SignalError::execution_checkpoint_stopped(stop),
            SignalError::ExecutionCheckpointStopped(expected)
        );
    }
}

#[test]
fn panic_and_storage_causes_remain_typed_and_distinct() {
    use crate::data::retained_storage::RetainedStoragePreparationDenial as Denial;
    assert_eq!(
        SignalError::execution_scope_denied(worth_execution::WorkCeilingDenial::Panicked),
        SignalError::ExecutionPanicked
    );
    for (denial, expected) in [
        (
            Denial::ChargeOverflow,
            SignalError::RetainedStorageChargeOverflow,
        ),
        (
            Denial::ChargeUnderflow,
            SignalError::RetainedStorageChargeUnderflow,
        ),
        (
            Denial::RetainedExtentHistoryUnavailable,
            SignalError::RetainedStorageHistoryUnavailable,
        ),
    ] {
        assert_eq!(SignalError::retained_storage_denied(denial), expected);
    }
}

#[test]
fn waiter_preparation_preserves_checkpoint_and_storage_causes() {
    use crate::data::graph::PendingRevalidationPreparationDenial;
    use crate::data::retained_storage::RetainedStoragePreparationDenial as Denial;
    for cause in [
        SignalCheckpointDenial::Cancelled,
        SignalCheckpointDenial::DeadlineElapsed,
        SignalCheckpointDenial::WorkCounterOverflow,
        SignalCheckpointDenial::WorkCeiling,
        SignalCheckpointDenial::NestedStopped,
    ] {
        assert_eq!(
            PendingRevalidationPreparationDenial::Storage(Denial::ExecutionStopped(cause))
                .into_signal_error(),
            SignalError::ExecutionCheckpointStopped(cause),
        );
    }
    for (denial, expected) in [
        (
            Denial::ChargeOverflow,
            SignalError::RetainedStorageChargeOverflow,
        ),
        (
            Denial::ChargeUnderflow,
            SignalError::RetainedStorageChargeUnderflow,
        ),
        (
            Denial::RetainedExtentHistoryUnavailable,
            SignalError::RetainedStorageHistoryUnavailable,
        ),
    ] {
        assert_eq!(
            PendingRevalidationPreparationDenial::Storage(denial).into_signal_error(),
            expected
        );
    }
}

#[test]
fn nested_performed_stop_keeps_its_publication_witness() {
    use crate::data::error::{
        SignalExecutionStop, SignalPublicationDisposition, SignalPublicationProgress,
    };
    use worth_foundational::{
        ExecutionPhysicalReport, ExecutionPosture, ExecutionReport, PartitionIdentity,
    };
    let identity = PartitionIdentity::new(3);
    let report = ExecutionReport::new(
        ExecutionPosture::Serial,
        17,
        17,
        ExecutionPhysicalReport::default(),
    );
    for performed in [false, true] {
        let mut progress = SignalPublicationProgress::default();
        progress.stopped_epoch(SignalPublicationDisposition::WorkerLocal);
        if performed {
            progress.complete_epoch(1);
        }
        let inner = SignalExecutionStop::new(
            SignalExecutionStopReason::Failure {
                identity,
                cause: SignalExecutionFailure::Cancelled,
            },
            Some(identity),
            progress,
            report,
        );
        let translated: SignalExecutionStopReason = MapStop::Failure {
            identity: PartitionIdentity::new(4),
            cause: MapKernelFailure::Domain(SignalError::execution_stopped(inner.clone())),
        }
        .into();
        if performed {
            let SignalExecutionStopReason::Failure {
                cause: SignalExecutionFailure::Domain(error),
                ..
            } = translated
            else {
                panic!("a performed effect must keep its witness");
            };
            assert_eq!(*error, SignalError::execution_stopped(inner));
        } else {
            assert_eq!(&translated, inner.reason());
        }
    }
}
