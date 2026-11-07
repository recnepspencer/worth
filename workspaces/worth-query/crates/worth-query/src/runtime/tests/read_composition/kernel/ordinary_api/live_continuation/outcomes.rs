use super::*;

pub(super) fn checkpointed(
    outcome: WorthQueryManagedLiveCheckpointOutcome,
) -> crate::ordinary::live::WorthQueryManagedLiveContinuation {
    match outcome {
        WorthQueryManagedLiveCheckpointOutcome::Checkpointed(completion) => {
            completion.into_continuation()
        }
        WorthQueryManagedLiveCheckpointOutcome::Stopped(stop) => {
            panic!(
                "managed checkpoint unexpectedly stopped: {:?}",
                stop.error()
            )
        }
    }
}

pub(super) fn resumed(
    outcome: WorthQueryManagedLiveResumeOutcome,
) -> crate::ordinary::live::WorthQueryManagedLiveResumeCompletion {
    match outcome {
        WorthQueryManagedLiveResumeOutcome::Resumed(completion) => completion,
        WorthQueryManagedLiveResumeOutcome::Stopped(stop) => {
            panic!("managed resume unexpectedly stopped: {:?}", stop.kind())
        }
    }
}

pub(super) fn stopped(
    outcome: WorthQueryManagedLiveResumeOutcome,
) -> crate::ordinary::live::WorthQueryManagedLiveResumeStop {
    match outcome {
        WorthQueryManagedLiveResumeOutcome::Stopped(stop) => stop,
        WorthQueryManagedLiveResumeOutcome::Resumed(_) => {
            panic!("hostile managed resume unexpectedly succeeded")
        }
    }
}
